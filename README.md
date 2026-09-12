# greybox

A learning project. I'm learning Rust (coming from years of Python/OOP, and
some parallel experimenting with Odin) by building a point-and-click
adventure engine from scratch with [Raylib](https://www.raylib.com/)
([raylib-rs](https://github.com/raylib-rs/raylib-rs) bindings).

Long-term reference points for the kind of game this engine should support:
_Broken Sword_, _Full Throttle_, _3 Skulls of the Toltecs_. Think scenes,
walkable characters, inventory, dialogs, simple menus, save/load, cutscenes,
basic audio.

There is no game design yet — no story, no assets. Everything is built
MVP-first: one scene, one character, one action, one dialog at a time, using
geometric primitives (rectangles, circles) as stand-ins for art that doesn't
exist yet. Primitives get swapped for real sprites/backgrounds later without
changing how the engine works.
We keep the visual debug available.

This is not a portfolio piece and not optimizing for feature count — it's
optimizing for learning idiomatic, efficient Rust. See `AGENTS.md` if you're
an AI agent working in this repo.

## Prerequisites (Linux)

`raylib-rs` builds Raylib from source, so you need `cmake` and a C
toolchain, plus the usual windowing/audio dev libraries.

Arch:

```bash
cmake alsa-lib libx11 libxrandr libxinerama libxcursor libxi mesa
```

## Running

```bash
cargo run
```

## Testing

```bash
cargo test
```

Not everything is unit-tested — only the parts where a test earns its
keep (real logic, edge cases), not getters/setters or glue code.

## Benchmarking

Hot paths get a [criterion](https://github.com/bheisler/criterion.rs)
benchmark once there's an actual hot path to measure — not before. None
exist yet.

## CLI-driven play (planned)

Once the engine can run a scene, the goal is to make it drivable from the
command line (move here, interact with X, combine A with B) so playthroughs
can be scripted and used as an automated smoke test, not just played by
hand. Not built yet.
This is supposed to be eventually tested by the AI agents to see how difficult
the game is (once it is developed).

## License

MIT, see `LICENSE`.
