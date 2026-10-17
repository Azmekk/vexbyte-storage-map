# Vexbyte Storage Map

A fast, good-looking disk space explorer for Windows with a built-in duplicate finder for your personal files.

**VSM** scans a drive or folder and shows you where the space goes: a sortable tree, a zoomable treemap, a sunburst, a file type breakdown and a largest-files list. On NTFS drives it can read the Master File Table directly for scans that finish in seconds. The duplicate finder looks through your Pictures, Videos, Music, Documents and Downloads for exact copies and lets you review them with thumbnails before anything moves to the Recycle Bin.

## Features

- Two scan methods: a parallel folder walk that works everywhere, and an MFT reader for NTFS drives (admin required) that is many times faster.
- Treemap and sunburst visualizations rendered on canvas, with animated zoom.
- Virtualized tree table, breadcrumbs, inspector panel and `Ctrl+K` search.
- Duplicate finder for images, videos, audio, PDFs, Office documents and archives, with size grouping, partial hashing and BLAKE3 full hashing.
- Keep rules (oldest, newest, preferred folder, shortest path) and a review screen with shell thumbnails.
- Everything destructive goes to the Recycle Bin and is undoable. Cloud placeholder files are never downloaded.
- Dark and light themes, reduced-motion support.

## Tech

Tauri 2, Rust (edition 2024), React 19, TypeScript, Tailwind CSS v4, shadcn/ui, d3-hierarchy, Motion, Zustand, Biome, Vitest.

## Building

Requirements: Rust 1.99+, Node 24+, pnpm, Visual Studio Build Tools with the C++ workload, WebView2 (ships with Windows 11).

```bash
pnpm install
pnpm tauri dev
```

Release build and installer:

```bash
pnpm tauri build
```

Checks:

```bash
pnpm check && pnpm typecheck && pnpm test
cargo fmt --check && cargo clippy --workspace --all-targets && cargo test --workspace
```

## Releases

Pushing a tag such as `v0.1.0` builds the app and publishes the NSIS installer as a GitHub release.

## Plan

The full implementation plan lives in [PLAN.md](PLAN.md).

## License

MIT
