# Scene system — design notes

Status: design agreed, nothing implemented yet. Reference games: Broken Sword,
Full Throttle, 3 Skulls of the Toltecs.

**What implementing this gets you:** a working adventure engine — scenes with
backgrounds, a character who walks around a hand-authored walkable area, hotspots
answering four verbs differently depending on story flags, sequenced actions with
walk-to behaviour, scene transitions, stationary NPCs, depth scaling and draw
order, and a tool to author it all. Every piece is greyboxed with coloured
rectangles.

**What it does not get you:** a game. No story, no art, no dialogue, no inventory
UI, no menu, no audio, no animation, no save/load. Those are deliberately deferred
and each has a hook waiting — but none of them is designed yet, and the engine
being "done" per this document is not the same as the game being playable in any
meaningful sense.

## State separation

- `GameState` — persistent world state: flags, `visited` per scene, inventory. The *only* thing that gets saved. Save/load later.
- Scene **data** — static, immutable: background, walkable polygon, hotspots, NPC placements, props. No game state.
- Scene **runtime instance** — built on scene entry from the data (NPC positions, animation timers), discarded on exit. Nothing here persists.
- `Interaction` — transient UI state: selected verb, held inventory item, pending intent. Lives next to `GameState`, not inside `Scene` (held item and selected verb survive scene transitions).

## Verbs, handlers, conditions

- `Verb` enum, fixed: Look, Use, Talk, Take.
- **A hotspot's shape is a polygon — the same polygon type as the walkable area.** One geometry type, one point-in-polygon test, one validator, three uses (walkable area, hotspots, later NPC bounds). Rectangles would be less work now and wrong the first time an object isn't rectangular, which in these games is immediately.
- A handler is: verb + guard conditions (AND) + a sequence of actions.
- A hotspot holds a list of handlers. **First handler whose verb matches and whose guards pass wins.** Handler order *is* the else-chain — a locked door is `Use`+[has key] → unlock, then `Use`+[] → "It's locked."
- No conditions inside a sequence. Keeps `Action` non-recursive and flat, which keeps the literal-Rust export readable. Accepted limit: no branching mid-sequence; split into two handlers if it ever comes up.
- Falls back to a per-scene default response (`Option`), then to a global one.
- Item combos: same pattern on `Item`, keyed by `ItemId`. Nothing more general.
- The same guard mechanism is reused for NPC presence and scene-enter handlers.

## Actions

- `Action` is pure data: `ShowText`, `SetFlag`, `GoToScene`, `GiveItem`, `RemoveItem`, `WalkTo(point, facing)`, `StartDialogue(DialogueId)`.
- Actions take time (`ShowText` holds for N seconds, `WalkTo` takes frames), so a sequence cannot run in one frame. A small runner holds the sequence, an index, and the current action's state; each frame it ticks and the action reports done/running. This is also where cutscenes and dialogue plug in later. Keep it a `match` — no traits.
- If scene data is `&'static`, the runner holds the slice + an index. If scenes end up as `Vec`, it must clone the sequence instead (`Action` is small).

**Control rules while a sequence runs** — these are behaviour decisions, not implementation details, and all three have to be answered or the runner is underspecified:

- **Input is locked.** While the runner is active, clicks do not start a new intent — that's what makes a sequence feel like a scripted beat rather than something you can walk out of mid-sentence. The one exception is skipping (below).
- **A click skips the current `ShowText`** rather than being ignored outright, which is how all three reference games behave. So the runner has to accept input, not just `dt`. Design its tick signature for that from the start.
- **`GoToScene` aborts the rest of the sequence.** The scene it referred to is gone and any following `WalkTo` or hotspot reference is meaningless. Make it terminal by rule, not by accident.

This also settles what `AppState::Cutscene` is for: **almost nothing**. A scripted beat inside a scene is just the runner with input locked, not a separate app state. Keep `Cutscene` only if there turns out to be a genuinely different mode (full-screen, no scene, no player). Until then it's a variant with no distinct behaviour — don't write it.

## Walk-to and pending intent

- A **handler** carries an optional walk-to point + facing — per (hotspot, verb), because you look at a door from further away than you open it. Not per action: `SetFlag` has no position.
- Mid-sequence movement is the explicit `WalkTo` action, not an implicit position on every action.
- On click: `Interaction` stores the **intent** (hotspot + verb). The player walks; on arrival the handler is **resolved then** (not at click time) — the intent is small stable data and resolution stays a pure function of (scene, game state, intent). A new click cancels a pending intent.
- No walk-to point (`None`) → runs immediately. Inventory item combining never walks.
- `Player` needs a facing/direction — `PlayerState` currently has none.

## NPCs

- NPCs do not move during normal play; several can be present at once with independent *behaviour* (which dialogue, whether present, how they react) — but that is all conditions over `GameState`, not movement. No actor system, no trait objects.
- Scene data: placement (position, sprite, walk-to point, handlers, dialogue id) + a presence condition using the same guards.
- Animation timers and scripted cutscene movement live in the scene runtime instance and are thrown away on exit.

## Walkable area and pathfinding

- One **concave** polygon per scene. A pillar you walk around is expressed by leading the outline in through a narrow channel, around the pillar, and back out ("keyhole") — so no holes list, no even-odd rule, format stays simple. The channel must have a few pixels of real width; a zero-width slit is a degenerate polygon and breaks point-in-polygon and segment tests on coincident edges. Put it against a wall.
- Pathfinding: **visibility graph over the reflex vertices** + A*. Nodes are start, goal and the concave corners; an edge exists if the segment between two nodes lies entirely inside the polygon. Paths come out taut, so no funnel smoothing is needed (the reason a single polygon beats walkboxes here).
- The fiddly part, and where the bugs will be: "segment lies inside the polygon" is *not* just "crosses no edge" — a segment can leave through a notch without a proper crossing. Test proper edge crossings **and** that the midpoint is inside. This one deserves a unit test.
- `f32` is not `Ord` — `total_cmp` for the A* open set (`BinaryHeap` + `Reverse`).
- Click outside the polygon: clamp to the nearest point on the boundary. Note this is a *different* layer from the letterbox `None` — screen→scene rejects clicks on the black bars, and only then does the walkable check clamp a valid scene point. Two mechanisms, two levels; don't merge them.
- **`Player` has to follow a path, not a point.** `PlayerState::Walking { target }` currently holds a single `Vector2`; pathfinding turns that into a list of waypoints plus an index, advancing on arrival at each. This is a change to existing code, not new code, and it is the moment `move_towards` stops being the whole movement system.

## Depth: scale and draw order

- The sort/scale key is the **y of the character's feet** (the existing bottom-centre origin in `Player::draw`). One number drives both how large the character is drawn and when it is drawn.
- Scale: interpolate between a near and a far scale over the walkable area's y range.
- Draw order: build a small per-frame list of `(key, what)` where `what` is an **enum** (player / NPC index / prop index), sort it, draw it. Keep the `Vec` in render state and `clear()` it each frame so capacity is reused.
- Background and foreground overlay are not in the sort — always first and last.
- Explicitly rejected: a `GameObject` trait with `update`/`draw`. Update has nothing to unify (a prop does nothing, an NPC ticks a timer, the player paths); only *draw order* needs unifying, and the drawable set is closed → enum, not trait object.
- Escape hatch for a long counter or a staircase, when it comes up: an integer layer on the prop, sort by (layer, y). Not now.

## Scene entry

- A scene holds a list of enter-handlers, same guards as hotspots.
- "First time here" is `visited: [bool; SceneId::COUNT]` in `GameState`, set automatically, with a `FirstVisit` guard — not a hand-written flag per scene.
- **Set `visited` after evaluating the enter guards**, or `FirstVisit` never passes. Worth a test.

## Content validation

- A public `validate` in the lib, **not** logic living inside a test: the
  integration test runs it over every `SceneId`, and the editor calls it live
  while drawing (red outline = you can't save this). One implementation, two
  consumers — same rule as the scene↔screen transform.
- Integration tests in `tests/` only see the lib target, not a binary — another
  reason for the lib. Pure geometry over `Vector2`, so they run headless.
- Checks on the walkable polygon:
  - fewer than 3 vertices; zero-length edges (repeated vertex)
  - self-intersection of non-adjacent edges
  - **collinear overlap of two edges** — a zero-width keyhole shows up as this,
    *not* as a crossing, so a plain "proper intersection" test misses it. Same
    class of bug as the segment-inside-polygon test in pathfinding: degenerate
    and collinear cases are the only place bugs live in this whole area.
  - **minimum channel width**: closest distance between any pair of non-adjacent
    edges must exceed a few pixels. Catches near-degenerate shapes that a pure
    intersection test passes. O(n²) over ~30 vertices is nothing.
  - **consistent winding (CW vs CCW)**. Reflex vertices are detected by the sign
    of a cross product, and that sign flips with the winding — a polygon drawn
    the other way round marks exactly the convex corners as concave, and A* then
    routes through walls while the polygon looks perfectly fine. Pick one
    winding, check it (sign of the area), and have the editor fix it silently.
- Same pass checks every walk-to point lies inside the polygon.

## Flags and IDs

- All IDs are enums: `SceneId`, `ItemId`, `FlagId`, `HotspotId`, `DialogueId`. Compile-time checked; recompiling to add content is accepted.
- Flags: `[bool; N]` indexed by `FlagId as usize`. Adding a flag is one enum variant and nothing else; save/load is serialising the array.
- Guard against `N` drifting from the enum with a unit test asserting the last variant's discriminant + 1 == `N`.

## App structure

- `enum AppState { Menu, InScene(..), Cutscene(..) }` — same idiom as `PlayerState`. No trait objects. `GameState` and `Interaction` live one level above it, not inside a variant.
- **Store `SceneId`, never a `&Scene`.** A struct holding both the scene table and a borrow into it is self-referential.
- **Effects out as data.** Update borrows `&Scene`/`&GameState` immutably and *returns* what happened; the caller applies it after the borrow ends. Frame becomes: gather input → update (`&`) → apply (`&mut`) → draw. The borrow checker never fights, and the logic is testable without a window.
- Overworld map is just another scene, not a special type.

## Coordinates

- Fixed virtual resolution; everything drawn in scene coordinates into a `RenderTexture2D`, then one `draw_texture_pro` to the window with letterbox. Chosen over `Camera2D` because it *forces* all game code into scene space, which is the invariant we want — and the editor gets it for free.
- Gotcha: the render texture is vertically flipped (OpenGL FBO) — the source rectangle needs a **negative height**.
- Gotcha: `set_texture_filter` with `POINT`, or scaling blurs.
- Exactly **one** function and its inverse, shared by drawing, hit-testing, player targeting and the editor. Not two implementations.

## Targets

- One package, three targets: `src/lib.rs` (all engine code), `src/main.rs` (game), `src/bin/editor.rs` (tools). Cargo auto-discovers all three; no `[[bin]]` sections needed, only `default-run` so plain `cargo run` stays unambiguous.
- Two `[[bin]]` targets *cannot* share modules — each would compile its own private copy with incompatible types. Hence the lib.
- Do this before hand-building scene #1, not at editor time.
- Shipping without the editor is free: separate binaries don't link each other, so `cargo build --release --bin greybox` never contained the editor code in the first place. The **lib is shared**, though — editor-only code (polygon drawing, Rust export, editor UI) belongs under `src/bin/editor/`, not in the lib. The lib holds only what both use: scene data, geometry, drawing, transform. A Cargo `editor` feature is the fallback if that ever stops being separable cleanly; premature now.
- Size, realistically: raylib statically links a C library, so a few MB is the floor, and embedded assets will later dwarf the code entirely. `[profile.release]` is set up with `strip` (biggest win by far) + LTO + `codegen-units = 1` + `panic = "abort"`. Deliberately *not* `opt-level = "z"` — a few hundred kB is a bad trade against runtime cost in a game.
- `default-run = "greybox"` is already in `Cargo.toml`, so plain `cargo run` stays unambiguous once the editor binary exists.

## Assets

- Dev: load from disk, so art tweaks don't trigger a recompile.
- Release (later): embed with `include_bytes!` so the shipped binary has no loose files.

## Roadmap

Each milestone is playable or demonstrable on its own; nothing here is a
refactor-only step except M0. The ordering rule throughout is the one in
`AGENTS.md`: smallest vertical slice first, breadth after.

**M0 — lib + binaries.** No behaviour change. *See Step 1 in detail.*
Done when `cargo build`/`run`/`test` all work and `main.rs` holds no game logic.

**M1 — scene↔screen transform.** *See Step 2 in detail.*
Done when resizing the window changes nothing about the game and clicks land under
the cursor at any size.

**M2 — coordinate picker** (`src/bin/editor.rs` v0). *See Step 3 in detail.*
Done when the same image feature clicked at two window sizes prints the same numbers.

**M3 — scene #1 renders and is walkable, with no pathfinding.**
Hand-written scene data: background, walkable polygon, point-in-polygon, click to
walk with the existing straight-line `move_towards`, clamping clicks to the polygon.
Draw the polygon as a debug overlay.
Done when you can walk around inside the shape and not outside it.

**M4 — the vertical slice: one hotspot, one verb, one action.**
Hotspot polygon, hit-testing, a hardcoded verb (no verb-coin yet), one handler with
a walk-to point, the action runner, `ShowText`. Click the hotspot → the player walks
there → text appears.
**This is the milestone that proves the architecture.** Everything before it is
plumbing; everything after it is breadth. If the handler/intent/runner design is
wrong, it shows up here — which is exactly why it comes before pathfinding, the
verb-coin, and the editor.

**M5 — guards and flags.** `GameState`, `FlagId`, the `[bool; N]` array, guard
conditions, first-match-wins, scene-level and global default responses.
Done when a hotspot answers differently before and after a flag is set — i.e. the
locked-door example from this document actually runs.

**M6 — verb-coin.** The `Verb` enum wired to real input, plus whatever invocation
gesture you settle on (currently an open TODO).
Done when Look and Use on the same hotspot give different answers.

**M7 — a second scene.** `SceneId`, the scene table, `GoToScene`, enter-handlers,
`visited` and `FirstVisit`.
Done when you can walk between two scenes and the first-visit text only fires once.

**M8 — pathfinding.** Visibility graph + A*, `Player` following a waypoint list,
the concave polygon and its validator.
Deliberately this late: it's a self-contained chunk that proves nothing about the
architecture, and M3–M7 work fine with straight-line movement in a roughly convex
test shape.

**M9 — depth.** Feet-y scaling, the sorted draw list, a foreground overlay, props.
Done when the player can walk behind something and shrinks with distance.

**M10 — NPCs.** Placement in scene data, presence guards, the scene runtime instance.

**M11 — the editor proper.** Grown from M2: multiple named polygons, live validation
(the validator from M8), export as literal Rust source, test mode that calls the
game's own draw with a hotspot overlay.
Deliberately last of the engine work: by now the data format it targets has been
validated by hand-writing four or five scenes, so the editor is built against
something known to be right rather than guessed at.

**Beyond** — inventory UI, dialogue, save/load, menu, sprites and animation, audio.
All deferred by design; see the Deferred section for what already has a hook.

## Step 1 in detail — lib + binaries

Goal: make code shareable between game and editor. No behaviour change; the
window must look and act exactly as it does now when this is done.

**Layout**

- `src/lib.rs` — declares the modules, nothing else. Starts as just `player`,
  grows to `view` (step 2), then `geom`, `scene`, `game_state`, `action`.
- `src/main.rs` — stays thin on purpose: raylib init, window config, the loop
  skeleton, and calls into the lib. Nothing that the editor would also want.
- `src/bin/editor.rs` — added in step 3.
- Cargo discovers all three automatically. `Cargo.toml` is already set up
  (`default-run`), nothing more to add.

**The one thing that will bite**

Visibility changes meaning. In a binary, `pub` is nearly decorative — everything
can see everything within the crate. In a lib, `pub` *is* the API surface: the
game and the editor are now separate crates consuming it, and anything not `pub`
is invisible to them. Expect a wave of privacy errors on the first build. Don't
blanket-`pub` your way out of it — each one is a real question about whether that
item is part of the engine's interface. `Player`'s fields and `PlayerState`
should stay private; `new`/`update`/`draw` are the interface.

**Done when**

- `cargo build`, `cargo run` and `cargo test` all work.
- `main.rs` contains no game logic — only window setup and the loop.

## Step 2 in detail — scene↔screen transform

The most important step of the three: everything downstream (hit-testing,
picker output, hotspots, editor) depends on it, and getting it wrong means
silently wrong coordinates rather than a crash.

**The model**

Pick one virtual resolution as a constant and draw *everything* in those
coordinates. 640×480 is the sensible start — it matches the current window, is
period-correct for Broken Sword, and the whole point is that the number lives in
one place and can change later. Render the frame into a `RenderTexture2D` of that
size, then blit it once to the window, scaled and centred, with black bars on
whichever axis doesn't fit.

The window then has no say over game coordinates at all. That is the invariant.

**The maths**

- `scale = min(window_w / virtual_w, window_h / virtual_h)`
- `offset = (window_size - virtual_size * scale) / 2` — the letterbox bars
- screen → scene: `(screen - offset) / scale`
- Recompute both every frame from the current window size. It's two divisions;
  don't build resize-event handling for it.

Represent it as a small value type holding `scale` and `offset`, with a method
each way. Not a trait, not stored globally — construct it from the window size at
the top of the frame and pass it where it's needed.

**Decisions to make while writing it**

- *Clicks on the black bars*: screen→scene should return an `Option` and give
  `None` outside the letterboxed area. Clicking a bar must not move the player.
  Clamping would silently teleport him to the edge instead — worse.
- *Fractional vs integer scale*: fractional fills more of the window but makes
  pixels unevenly sized; integer (floor the scale) is perfectly uniform but wastes
  border. Irrelevant while everything is coloured rectangles, visible the moment
  real pixel art lands. Start fractional, keep it a one-line change.

**raylib gotchas, all verified against raylib-rs 6.0**

- `rl.load_render_texture(&thread, w, h)` returns a `Result` and the texture must
  outlive every frame — create it once after window init, keep it in a local.
- `begin_texture_mode(&thread, &mut rt)` and `begin_drawing(&thread)` both return
  RAII scope handles that each borrow `rl` mutably. They cannot overlap — put the
  texture-mode pass in its own block so it drops before `begin_drawing` starts.
  This is the borrow checker enforcing a real raylib rule, not an annoyance.
- **The render texture is vertically flipped** (it's an OpenGL FBO). The source
  rectangle passed to `draw_texture_pro` needs a **negative height**, or the whole
  game renders upside down.
- `set_texture_filter(&thread, POINT)` on the render texture's texture, or scaling
  blurs it.
- **Use `get_screen_width`/`get_screen_height`, not `get_render_*`.** The render
  variants are multiplied by DPI scale, while `get_mouse_position` is in screen
  space — mixing them makes clicks land in the wrong place on a HiDPI display and
  nowhere else, which is a miserable bug to reproduce.
- Set the resizable window flag at init, otherwise the letterboxing can't actually
  be tested. This also makes the fullscreen toggle in the TODO trivial later.

**Integration point**

`main.rs` currently feeds `get_mouse_position()` straight into `Player::update`.
After this step it must convert first, and skip the update when the result is
`None`. That single change is what proves the transform is wired in.

**Tests (mine to write)**

Pure maths over (window size, virtual size) — no window needed, so these are fast
unit tests, not integration tests:

- round-trip: scene → screen → scene is the identity (within epsilon) for points
  inside the area
- screen → scene returns `None` on the bars
- centring: the virtual rect's centre maps to the window's centre at any window
  size
- extreme aspect ratios (very wide, very tall) produce bars on the expected axis

**Done when**

Resizing the window changes nothing about the game: the player rectangle keeps its
proportions and position relative to the scene, and a click lands exactly under the
cursor at any window size and aspect ratio.

## Step 3 in detail — coordinate picker

**Make this `src/bin/editor.rs` from the start.** The picker is the editor's first
version, not a throwaway — a separate `picker` binary would just be deleted later.

**What it does**

- Loads a background image from disk and draws it through the step-2 transform.
- Left click prints the point, in *scene* coordinates, to stdout as
  `Vector2 { x: .., y: .. },` — one per line, trailing comma, so the output pastes
  straight into an array. Struct literal, not `Vector2::new(..)`: `new` is not a
  `const fn` in raylib-rs 6.0, so only the literal form can live in a `const`.
- Draws a dot at each point and lines between consecutive ones, so the polygon is
  visible as it forms. Not editing — just feedback, and it's the difference
  between usable and unusable.
- Backspace removes the last point. The one concession to editing; without it,
  a misclick means starting over.
- **Diagnostics go to stderr, only the vectors to stdout**, so the output can be
  piped or selected cleanly.

**Not in this step**: saving, export to a file, named polygons, multiple polygons,
hotspots, validation. All of that is step 4.

**Prerequisite, and the only non-code blocker**

There is no art. Put any placeholder image in `assets/` — the picker needs
something with recognisable features to click on. Loading is relative to the
working directory, which `cargo run` sets to the package root, so `assets/bg.png`
works during development. A shipped binary won't have it, but that's what the
`include_bytes!` plan is for later.

**raylib note**

`rl.load_texture(&thread, path)` returns a `Result` and must be called after window
init; keep the `Texture2D` in a local for the program's lifetime.

**Done when**

Clicking the same recognisable feature of the image at two very different window
sizes prints the **same** coordinates. That is the real acceptance test for step 2,
and the reason the transform had to come first — a picker built against window
coordinates would produce numbers that quietly break on the first resize.

## Deferred — hooks exist, design later

- Inventory UI (opening it, picking an item, transferring it to `Interaction`). The data side is already in `GameState`.
- Dialogue system — `Action::StartDialogue(DialogueId)` is an opaque hook.
- Save/load — falls out almost free: `GameState` + current `SceneId`. That it comes out this simple is a good sign the separation is right.
- Localisation — English only for now, `ShowText` carries a `&'static str`. Swapping in a `TextId` later is mechanical.
- Debug UI for toggling flags in the editor (would need `FlagId` enumeration at runtime, e.g. `strum` or a hand-kept `ALL` array). Not needed yet.
- **Sprites and animation** — entirely undesigned. Everything is coloured rectangles, and the plan assumes that throughout: the only trace is the "animation timers" in the scene runtime instance. Frames, idle/walk cycles and facing-dependent sprites are a whole subsystem that arrives with real art, not before.
- **Audio** — the README wants basic audio; nothing here touches it. It will slot in as `Action` variants and a scene-level ambience, but it isn't designed.
- **Menu** — `AppState::Menu` exists as a variant and nothing more.
- `HotspotId` is speculative right now: with CLI play deferred, nothing references a hotspot by id. Keep the enum, but it earns its keep only once something addresses hotspots from outside the scene (scripted play, a flag hiding a hotspot).

## Open TODO

- No fullscreen toggle.
- Editor must validate that every walk-to point lies inside the walkable polygon — otherwise the player never arrives and the action silently never fires.
- Overlapping hotspots: hit-test order undefined.
- Hotspot name shown under the cursor (Broken Sword style) — not designed.
- How the verb-coin is invoked (hold-click? right button?) — not decided.
