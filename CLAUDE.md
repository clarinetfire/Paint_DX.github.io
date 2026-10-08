# Project: paint-rs (Paint DX)

Rust rewrite of Windows XP Paint for Windows 11, with layers, frames,
PNG-sequence and sprite-sheet export. Spec: docs/xp-paint.md and plan.md.

## Commands
- `cargo build --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo fmt --all -- --check`
- `cargo run -p paint-app`

## Rules
- `paint-core` has no UI, no OS calls, and no `unsafe` code.
- Pixel behaviour changes need a golden PNG in `tests/golden/`.
- Match XP behaviour from `docs/xp-paint.md` unless the spec explicitly says otherwise.
- Anti-aliasing is off by default to match classic pixel fidelity.
- Ask before adding an external dependency; note why in the commit message.
- Execute one milestone per session, then commit.
- Never copy Microsoft icons, strings or proprietary resources.
