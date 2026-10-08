# Agent Guidelines: Paint DX (paint-rs)

This project is a Rust rewrite of Windows XP Paint for Windows 11, with layers, frames,
PNG-sequence and sprite-sheet export. See `docs/xp-paint.md` and `plan.md`.

## Workspace Structure
- `crates/paint-core`: Core data structures and pixel algorithms (pure Rust, `#![forbid(unsafe_code)]`, no UI/OS).
- `crates/paint-app`: Desktop UI application using `eframe` (egui) and Windows platform integration.
- `tests/golden`: Golden image tests for non-regression pixel comparison.
- `docs/xp-paint.md`: Reference documentation for classic XP Paint functionality.

## Development Commands
- Build workspace: `cargo build --workspace`
- Test workspace: `cargo test --workspace`
- Check lints: `cargo clippy --workspace -- -D warnings`
- Code format: `cargo fmt --all -- --check`
- Run desktop app: `cargo run -p paint-app`

## Strict Conventions
1. **Purity of paint-core**: Never add UI dependencies (egui/eframe) or platform-specific syscalls to `paint-core`.
2. **Visual Fidelity**: Anti-aliasing must be disabled by default (XP crisp pixel rendering).
3. **Golden Testing**: Every drawing tool and filter must be backed by unit or golden-image tests.
4. **Clean Code**: Zero clippy warnings allowed on PRs/commits.
