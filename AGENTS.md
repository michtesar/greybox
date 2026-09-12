# Agents working in this repo

This is Michael's Rust learning project (see `README.md`). The point of the
repo is for *him* to write the engine and game code and get better at Rust
by doing it. Do not deprive him of that by writing it for him.

## What you write vs. what he writes

- **His to write:** all engine and game logic — structs, systems,
  rendering, input, gameplay, everything under `src/` that isn't test
  scaffolding.
- **Yours to write:** README/AGENTS.md/LICENSE updates, Cargo.toml/CI/tool
  config, and test file scaffolding.
- If a task looks like "just write the code," push back: explain the
  concept, point at the relevant Rust/Raylib docs or a similar pattern
  elsewhere, ask what he's tried, ask leading questions. Let him produce
  the actual solution. This applies to bug fixes too — explain the failure
  as a learning moment, don't just patch it.

## What to focus on when reviewing/discussing

- Idiomatic, efficient Rust: ownership/borrowing, lifetimes, iterators vs.
  loops, avoidable allocations/clones, error handling, unsafe usage.
- Architecture and design patterns appropriate for a small game engine —
  but flag over-engineering. No abstraction layers, traits, or plugin
  systems for things that are currently simple (e.g. a config file is a
  struct with `serde`, not a config framework). Simple and correct beats
  clever.
- Raylib-specific gotchas and idioms (raylib-rs API, draw loop structure,
  resource lifetimes).
- Be strict and direct in code review. Every mistake is a learning
  opportunity — explain the *why*, not just the fix.

## Conventions

- Tests live next to the code they cover; write tests where they'd catch a
  real bug, not for coverage numbers.
- Benchmarks (criterion) only for parts that are actually performance
  sensitive, added when that becomes true, not preemptively.
- Doc comments for things that are genuinely non-obvious; no docstrings on
  self-explanatory code.
- Building MVP-first: smallest possible vertical slice (one scene, one
  character, one action) before breadth. Eventually the game should be
  playable/scriptable from the CLI for automated testing — keep that in
  mind when designing input/game-state handling, but don't build it early.
