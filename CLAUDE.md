# Vexbyte Storage Map

Windows-only disk space explorer and duplicate finder. Tauri 2 + Rust (edition 2024) + React 19 + TypeScript.
The full design is in `PLAN.md`. Read sections 3 (code style), 4 (layout) and 15 (safety) before writing code.

## Non-negotiable safety rules

- Scanning is read-only. Never open a volume or file for write. Never open cloud placeholder files.
- No automated test, script or dev check may delete, move, rename or modify anything outside a temp
  directory the test itself created under `std::env::temp_dir()`. Clean up what you create.
- Never call the Recycle Bin, `IFileOperation`, `DeleteFile`, `RemoveDirectory` or any destructive
  shell API from a test, a dev script or an agent-driven check. Destructive paths are tested only by
  the maintainer by hand. Unit-test the planning logic around them instead.
- Never run `pnpm tauri build` or `cargo run` against the user's real folders in a way that changes them.

## Style summary

- Descriptive names, no abbreviations (except id, url, io, ui, mft, ntfs). Comments only for Win32/NTFS quirks,
  `// SAFETY:` notes and non-obvious performance choices.
- One responsibility per file. No `utils`, `helpers`, `misc`.
- Rust: `thiserror` enums per module, newtypes for ids/indices, all `unsafe` and Win32 calls inside a
  `windows_api` module per crate, `#![deny(clippy::unwrap_used, clippy::expect_used)]` in library crates
  (tests exempt), `cargo clippy -- -W clippy::pedantic` clean.
- TypeScript: strict, named exports only, one component per file (PascalCase), hooks hold logic,
  no direct `invoke`, everything via `src/lib/backend/`. No `any`.
- Biome (not ESLint/Prettier): 2 spaces, double quotes, width 100.

## Commands

```
pnpm install
pnpm tauri dev            # run the app
pnpm check                # biome ci
pnpm typecheck            # tsc --noEmit
pnpm test                 # vitest run
pnpm bindings             # regenerate src/lib/backend/generated.ts after changing commands or IPC types
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Crates in `crates/` must build and test without Tauri. `src-tauri` is a thin adapter only.

`site/` is the public download page: a separate static Vite + React project with its own
`package.json` (run `pnpm install`, `pnpm dev`, `pnpm test` inside it). See `site/README.md`.
