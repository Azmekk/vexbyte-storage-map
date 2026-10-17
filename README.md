# Vexbyte Storage Map

A fast, good-looking disk space explorer for Windows with a built-in duplicate finder for your personal files.

**VSM** scans a drive or folder and shows you where the space goes: a sortable tree, a zoomable treemap, a sunburst, a file type breakdown and a largest-files list. On NTFS drives it can read the Master File Table directly for scans that finish in seconds. The duplicate finder looks through your Pictures, Videos, Music, Documents and Downloads for exact copies and lets you review them with thumbnails before anything moves to the Recycle Bin.

**Status:** 0.1.0 is the first pre-release. Windows 10 and 11 (64-bit) only.

## Screenshots

![Home screen with drive cards](docs/screenshots/home-dark.png)

![Explorer with the treemap of a scanned folder](docs/screenshots/treemap-dark.png)

![Sunburst coloured by file type](docs/screenshots/sunburst-dark.png)

![Duplicate finder results with keep and remove choices](docs/screenshots/duplicates-results-dark.png)

![Global file search across every indexed drive](docs/screenshots/search-dark.png)

## Features

- Two scan methods: a parallel folder walk that works everywhere, and an MFT reader for NTFS drives (admin required) that is many times faster.
- Treemap and sunburst visualizations rendered on canvas, with animated zoom.
- Virtualized tree table, breadcrumbs, inspector panel and `Ctrl+K` search.
- Search: find any file or folder on every local drive by name or path in milliseconds, in the
  spirit of Everything (`Ctrl+Shift+F` or `F3`). The index is built in the background when the
  app starts and kept in memory.
- Duplicate finder for images, videos, audio, PDFs, Office documents and archives, with size grouping, partial hashing and BLAKE3 full hashing.
- Keep rules (oldest, newest, preferred folder, shortest path) and a review screen with shell thumbnails.
- Move duplicates to the Recycle Bin one file at a time, one group at a time, or every marked copy at once, always after a confirmation that lists exactly what moves.
- Everything destructive goes to the Recycle Bin and is undoable. Cloud placeholder files are never downloaded.
- OneDrive and other online-only files count only what is downloaded. Files with no data on this PC are hidden from the tree and charts until you choose to show them, so a folder never looks large because of files that are not on the disk.
- Dark and light themes, reduced-motion support, keyboard shortcuts (press `?` in the app).

## Search syntax

The Search screen understands a subset of Everything's syntax: words that must all appear in the
name (`annual report`), quoted phrases, wildcards (`*.iso`, `img_????.png`), path terms
(`photos\2024`) and the filters `ext:`, `size:`, `type:`, `file:` and `folder:`, for example
`holiday ext:jpg;png size:>2mb`. Click **Search syntax** in the app for examples, or read the full
reference in [docs/SEARCH.md](docs/SEARCH.md).

## Safety

Scans are read-only: both scan methods open drives and folders for reading only, and cloud
placeholder files (OneDrive "online-only" files) are never opened, so nothing gets downloaded.
They are also never counted as space: a placeholder adds only the bytes it has on this disk to
its folder, and placeholders without any local data stay out of the tree unless you show them.
Streamed cloud drives such as Google Drive for desktop are recognized and never scanned or searched
for duplicates, because reading them would download their files. The
only action that changes anything on disk is **Move to Recycle Bin**, from the explorer or the
duplicates screen. It always shows a confirmation dialog listing exactly what will move, the
duplicate finder re-checks every file right before moving it and always keeps at least one copy,
and everything moved can be restored from the Windows Recycle Bin.

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

The installer lands in `target/release/bundle/nsis/` (the workspace target folder).

Checks:

```bash
pnpm check && pnpm typecheck && pnpm test
cargo fmt --check && cargo clippy --workspace --all-targets && cargo test --workspace
```

Before a release, walk through the [manual test checklist](docs/MANUAL_TESTING.md): both scan
methods including the elevated relaunch, OneDrive placeholders, exFAT drives, a duplicate search on
a hard disk and the Recycle Bin flows on a temporary folder.

## Icon

The app icon is a placeholder. To replace it, swap `src-tauri/icons/icon-source.png` (1024 × 1024
PNG) and regenerate the sizes with `pnpm tauri icon src-tauri/icons/icon-source.png`; also update
`public/favicon.png`, which the sidebar shows.

## Releases

Pushing a tag such as `v0.1.0` runs the frontend checks, builds the app and publishes the NSIS
installer as a GitHub release named "Vexbyte Storage Map v0.1.0" (0.x versions are marked as
pre-releases). Builds are unsigned; Authenticode signing can be added with
`bundle.windows.certificateThumbprint` or `bundle.windows.signCommand` in
`src-tauri/tauri.conf.json`.

## Plan

The full implementation plan lives in [PLAN.md](PLAN.md).

## License

MIT
