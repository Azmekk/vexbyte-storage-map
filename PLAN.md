# Vexbyte Storage Map (VSM): Implementation Plan

A Windows disk space explorer in the spirit of WizTree / TreeSize, with a much better interface and a built-in duplicate finder for personal files: photos, videos, music, PDFs, Office documents and archives. Built with Tauri 2, Rust and React.

### Naming

| Where | Value |
|---|---|
| Product name (window title, installer, Start menu) | Vexbyte Storage Map |
| Short name (UI where space is tight, docs) | VSM |
| Tauri `productName` | `Vexbyte Storage Map` |
| Tauri `identifier` | `com.vexbyte.storagemap` |
| Tauri app crate | `vexbyte_storage_map` |
| npm package name | `vexbyte-storage-map` |
| Executable | `VexbyteStorageMap.exe` |
| App data and cache folder | `%LOCALAPPDATA%\Vexbyte\Storage Map\` |

The library crates in `crates/` keep their descriptive names (`disk_scanner`, `duplicate_finder`, `file_actions`); they describe what they do, not the product.

---

## 1. Scope

### In scope

1. **Disk viewer**
   - Scan a drive or folder and show where the space goes.
   - Two scan methods behind one interface:
     - **Folder walk**: works everywhere, no admin needed.
     - **MFT read**: reads the NTFS Master File Table directly. Requires admin and an NTFS volume. Much faster.
   - Sortable, virtualized tree; zoomable treemap; sunburst; file type breakdown; largest files list; search.
   - Actions: open, reveal in Explorer, copy path, move to Recycle Bin.

2. **Duplicate finder** for user files
   - The user picks folders (Pictures, Videos, Music, Documents, Downloads, Desktop, custom) and file categories with checkboxes:
     - **Images**: jpg, jpeg, png, gif, webp, heic, heif, bmp, tif, tiff, raw formats (cr2, cr3, nef, arw, dng)
     - **Videos**: mp4, mov, mkv, avi, wmv, webm, m4v, mts
     - **Audio**: mp3, flac, wav, m4a, aac, ogg, wma, opus
     - **PDFs**: pdf
     - **Documents**: doc, docx, odt, rtf, txt, md, epub
     - **Spreadsheets**: xls, xlsx, xlsm, ods, csv
     - **Presentations**: ppt, pptx, odp
     - **Archives & installers**: zip, rar, 7z, tar, gz, iso, exe, msi
   - Extension lists live in one place (`file_categories.rs`) so they are easy to extend.
   - Exact duplicates only.
   - Review screen with thumbnails, keep rules, and move to Recycle Bin.

### Out of scope

- Near-duplicate detection (similar images, audio fingerprinting).
- Cleanup recommendations / issue detection. Keep the architecture open to it, but build nothing for it now.
- macOS builds. Windows and Linux (x64, since 0.2.0) ship today. macOS may be added in the future (a distributable build needs a paid Apple Developer ID to sign and notarize it), so the code must stay easily portable to macOS; see section 16 (Portability rules). The download page counts interest through one GitHub issue.
- Scanning or deduplicating system folders. The duplicate finder never touches Program Files, Windows, AppData or similar.

---

## 2. Tech stack

### Backend

- Rust 1.99.1 (latest stable, already installed locally), edition 2024. Pin it with a `rust-toolchain.toml` at the repo root (`channel = "1.99.1"`).
- Tauri 2
- `windows` crate (windows-rs) for Win32 APIs
- `rayon` for parallel directory walking
- `blake3` for full file hashing, `xxhash-rust` (xxh3) for partial hashing
- `thiserror` for error types
- `specta` + `tauri-specta` to generate TypeScript bindings for every command and event type
- `tracing` for logging

### Frontend

- React 19 + TypeScript (strict mode)
- Vite 8.3.4
- Tailwind CSS v4
- shadcn/ui for all standard components (buttons, dialogs, tabs, tooltips, context menus, command palette, sidebar, cards, checkboxes)
- shadcn charts (Recharts) for standard charts: bars, donuts
- `d3-hierarchy` for treemap and sunburst layout math, rendered by our own components on `<canvas>`
- `@tanstack/react-virtual` for the tree table and long lists
- `motion` (Motion for React) for animations
- `zustand` for client state
- Biome 2.x for linting and formatting; no ESLint, no Prettier
- Vitest for unit tests

---

## 3. Code style rules

These rules matter as much as the features. Read them before writing code.

### General

- **Names explain intent.** A long descriptive name beats a short name plus a comment.
  - Bad: `fn proc(e: &E, f: u32) -> R` with a comment explaining it.
  - Good: `fn add_entry_to_parent_directory(entry: &DirectoryEntry, parent: NodeIndex) -> Result<NodeIndex, TreeBuildError>`
- **No cryptic nesting.** Do not write `ok(yes(try_do(op(x))))`. Break the work into named steps with named intermediate values.
- **Comments are rare.** Write them only for things the code cannot say. The usual cases are:
  - Win32 and NTFS quirks: "NTFS stores up to 4 names per file; the DOS 8.3 name must be skipped".
  - `// SAFETY:` notes on every `unsafe` block, which is standard Rust practice.
  - Non-obvious performance decisions.

  Never comment what a line obviously does.
- **One responsibility per file.** No `utils.rs`, `helpers.ts` or `misc` dumping grounds. If a helper has no clear home, the module structure is wrong.
- **No abbreviations** except universally known ones: `id`, `url`, `io`, `ui`, `mft`, `ntfs`.

### Rust specifics

- Follow standard Rust conventions; `cargo fmt` and `cargo clippy -- -W clippy::pedantic` must pass, with justified `#[allow]`s only.
- `#![deny(clippy::unwrap_used, clippy::expect_used)]` in library crates, except in tests.
- Use newtypes for indices and identifiers: `NodeIndex(u32)`, `NameId(u32)`, `FileReference(u64)`, `ScanId`. Never pass raw `u32`/`u64` around for these.
- Errors use `thiserror` enums per module, with descriptive variant names: `VolumeOpenFailed { volume: String, source: windows::core::Error }`.
- All `unsafe` and all raw Win32 calls live in a dedicated `windows_api` module per crate. They are wrapped in safe functions with descriptive names. The rest of the code never calls Win32 directly.
- Prefer small structs with clear methods over free functions with many parameters. If a function takes more than four parameters, introduce a struct.

Example of the expected style:

```rust
pub fn scan_volume_with_mft(
    volume: &VolumeHandle,
    progress: &ScanProgressReporter,
    cancellation: &CancellationToken,
) -> Result<FileTree, MftScanError> {
    let volume_layout = read_ntfs_volume_layout(volume)?;
    let mft_data_runs = read_mft_data_runs(volume, &volume_layout)?;

    let mut tree_builder = FileTreeBuilder::with_capacity(volume_layout.estimated_record_count());

    for record_chunk in MftChunkReader::new(volume, &mft_data_runs, &volume_layout) {
        cancellation.return_if_cancelled()?;

        let records = record_chunk?;
        add_mft_records_to_tree(&records, &mut tree_builder);
        progress.report_records_processed(records.len());
    }

    Ok(tree_builder.link_parents_and_aggregate_sizes())
}
```

### TypeScript / React specifics

- Named exports only, no default exports (except where a tool requires one).
- One component per file. The file name matches the component in PascalCase.
- Components render; hooks hold logic. No IPC calls or data transformation inside JSX components.
- Never call Tauri `invoke` directly. All backend access goes through the generated bindings wrapped in `src/lib/backend/`.
- No `any`. No non-null assertions (`!`) unless unavoidable and isolated.
- Prefer explicit names: `selectedNodeIndex`, not `sel`; `formatBytesAsReadableSize`, not `fmt`.

### Biome

- `biome.json` at the repo root. Use the recommended rules plus:
  - `noExplicitAny`: error
  - `useExhaustiveDependencies`: error
  - `noNonNullAssertion`: warn
  - `useNamingConvention`: on, with defaults
- Configure import organizing and the formatter: 2-space indent, double quotes, line width 100.
- Exclude `src/components/ui/` (shadcn generated code) and the generated bindings file from linting.
- Add scripts: `lint`, `format`, `check` (Biome `ci`), `typecheck`, `test`.

---

## 4. Repository layout

```
/
├─ PLAN.md
├─ biome.json
├─ package.json
├─ Cargo.toml                         # workspace
├─ crates/
│  ├─ disk_scanner/                   # pure Rust, no Tauri dependency
│  │  └─ src/
│  │     ├─ lib.rs
│  │     ├─ file_tree/
│  │     │  ├─ mod.rs
│  │     │  ├─ node.rs                # Node struct, NodeIndex, NodeKind, NodeFlags
│  │     │  ├─ name_storage.rs        # interned names, NameId
│  │     │  ├─ file_tree_builder.rs   # collects entries, links parents, aggregates sizes
│  │     │  ├─ hard_link_tracker.rs   # counts each FileReference once
│  │     │  └─ tree_queries.rs        # children sorted, largest files, search, type breakdown
│  │     ├─ folder_walk/
│  │     │  ├─ mod.rs
│  │     │  ├─ parallel_directory_walker.rs
│  │     │  └─ directory_entry.rs
│  │     ├─ mft/
│  │     │  ├─ mod.rs
│  │     │  ├─ volume_layout.rs       # FSCTL_GET_NTFS_VOLUME_DATA result
│  │     │  ├─ data_runs.rs           # decoding non-resident attribute runs
│  │     │  ├─ mft_chunk_reader.rs    # large sequential reads of the MFT
│  │     │  ├─ record_fixups.rs       # update sequence array fixups
│  │     │  ├─ file_record.rs         # record header, in-use flag, base record reference
│  │     │  ├─ attributes/
│  │     │  │  ├─ mod.rs
│  │     │  │  ├─ standard_information.rs
│  │     │  │  ├─ file_name.rs
│  │     │  │  ├─ data.rs
│  │     │  │  └─ attribute_list.rs
│  │     │  └─ mft_record_to_tree.rs
│  │     ├─ scan_method.rs            # enum ScanMethod { FolderWalk, MasterFileTable }
│  │     ├─ scan_progress.rs
│  │     ├─ cancellation.rs
│  │     ├─ volumes.rs                # list drives, file system type, free/total space
│  │     └─ windows_api/              # all unsafe Win32 wrappers
│  ├─ duplicate_finder/               # pure Rust, no Tauri dependency
│  │  └─ src/
│  │     ├─ lib.rs
│  │     ├─ search_settings.rs        # folders + file categories
│  │     ├─ file_categories.rs        # category → extension lists
│  │     ├─ candidate_collector.rs    # walk selected folders, filter, skip cloud placeholders
│  │     ├─ size_grouping.rs
│  │     ├─ partial_hashing.rs
│  │     ├─ full_hashing.rs
│  │     ├─ drive_kind.rs             # SSD vs HDD (seek penalty) → read strategy
│  │     ├─ duplicate_group.rs
│  │     ├─ keep_rules.rs
│  │     └─ windows_api/
│  └─ file_actions/                   # shared: recycle bin, reveal in Explorer, thumbnails
│     └─ src/
│        ├─ recycle_bin.rs
│        ├─ reveal_in_explorer.rs
│        └─ shell_thumbnails.rs
├─ src-tauri/                         # crate vexbyte_storage_map, thin adapter layer only
│  └─ src/
│     ├─ main.rs
│     ├─ app_state.rs                 # holds scans, duplicate searches
│     ├─ commands/
│     │  ├─ volume_commands.rs
│     │  ├─ scan_commands.rs
│     │  ├─ tree_commands.rs
│     │  ├─ duplicate_commands.rs
│     │  └─ file_action_commands.rs
│     ├─ elevation.rs                 # detect admin, relaunch elevated
│     └─ bindings_export.rs           # tauri-specta export to src/lib/backend/generated.ts
└─ src/
   ├─ main.tsx
   ├─ app/
   │  ├─ App.tsx
   │  ├─ AppLayout.tsx
   │  └─ ThemeProvider.tsx
   ├─ components/ui/                  # shadcn generated
   ├─ lib/
   │  ├─ backend/                     # generated.ts + thin typed wrappers
   │  ├─ formatting/                  # formatBytesAsReadableSize.ts, formatRelativeDate.ts
   │  └─ motion/                      # shared transition presets
   └─ features/
      ├─ volumes/
      ├─ scan/                        # start, progress screen, cancel
      ├─ explorer/                    # tree table, breadcrumbs, inspector, context menu
      ├─ visualizations/
      │  ├─ treemap/
      │  ├─ sunburst/
      │  └─ file_types/
      ├─ search/                      # command palette search
      └─ duplicates/
```

The rule: **crates in `crates/` know nothing about Tauri or the UI.** `src-tauri` only converts between Tauri and the crates. This keeps the core testable with plain `cargo test`.

---

## 5. Core data model (`disk_scanner::file_tree`)

The full tree **never** crosses to the frontend. It lives in Rust and the UI asks for small views of it.

- Store nodes in an arena: `Vec<Node>`, addressed by `NodeIndex(u32)`.
- `Node` fields:
  - `name: NameId`
  - `parent: Option<NodeIndex>`
  - `first_child` and `next_sibling` (`Option<NodeIndex>`), or a children range. Pick whichever keeps `Node` small.
  - `logical_size: u64` and `allocated_size: u64` (own size for files, aggregated for directories)
  - `file_count: u32` and `directory_count: u32` (aggregated)
  - `modified_time: u64` (FILETIME)
  - `kind: NodeKind` (File, Directory)
  - `flags: NodeFlags` (bitflags: ReparsePoint, CloudPlaceholder, Compressed, Sparse, Hidden, System, Inaccessible)
- Names are interned in `NameStorage`: one big `String` buffer plus offsets. File extensions are interned separately for fast type breakdowns.
- **Memory target:** under 64 bytes per node, excluding name bytes. Add a test that asserts `size_of::<Node>()`.
- **Hard links:** `HardLinkTracker` remembers each `FileReference` already counted. The second and later links appear in the tree but contribute zero size to totals, and are flagged so the UI can show "hard link, counted once."
- **Size aggregation** is a single post-order pass after the tree is complete.

---

## 6. Scan method A: folder walk

- Accept a root path. Normalize it to an extended-length path (`\\?\C:\...`) so long paths work.
- Walk in parallel with a `rayon` scope: each directory is a task that lists its entries and spawns tasks for subdirectories.
- List directories in bulk with `GetFileInformationByHandleEx` using `FileIdBothDirectoryInfo` on an opened directory handle. This returns name, sizes (end of file and allocation size), attributes, timestamps and file ID per entry, **without opening individual files**.
- Never follow reparse points (junctions, symlinks). Record them as leaf nodes with the ReparsePoint flag.
- **Never open or read cloud placeholder files.** Detect them by attributes (`FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS`, `FILE_ATTRIBUTE_RECALL_ON_OPEN`, `FILE_ATTRIBUTE_OFFLINE`). Count their logical size separately as "in cloud".
- Access-denied directories become nodes flagged Inaccessible. They do not fail the scan.
- Workers send batches of entries (for example 4,096 at a time) to the tree builder over a channel. Do not take a lock per entry.
- Report progress at most ~10 times per second: files found, directories found, bytes found, current path.
- Cancellation is checked between directories.

---

## 7. Scan method B: MFT read

Available only when the process is elevated **and** the volume is NTFS. Otherwise the UI offers the folder walk, with a "Restart as admin for fast scan" option on NTFS volumes.

Steps:

1. Open the volume (`\\.\C:`) with read access and shared read/write.
2. Call `FSCTL_GET_NTFS_VOLUME_DATA` for bytes per cluster, bytes per file record, MFT start cluster and MFT valid data length.
3. Read MFT record 0 (the `$MFT` file itself) and decode its `$DATA` attribute's data runs. The MFT can be fragmented; the data runs give its real location on disk.
4. Read the MFT **sequentially in large chunks** (start with 8 MB and make it configurable) following the data runs. Large sequential reads are the whole reason this is fast.
5. For each record:
   - Verify the `FILE` signature, apply the update sequence array fixups, and skip records that are not in use.
   - Parse only the attributes we need:
     - `$STANDARD_INFORMATION`: timestamps and file attribute flags.
     - `$FILE_NAME`: parent file reference, name and namespace. Skip DOS-only 8.3 names. A record can have several Win32 names; each is a hard link.
     - `$DATA`, unnamed stream only: logical and allocated size, for both resident and non-resident forms. Named streams (alternate data streams) are ignored in v1.
     - `$ATTRIBUTE_LIST`: when present, some attributes live in extension records. Handle this by merging extension records into their base record by base file reference.
   - Use the sequence number in file references to discard stale parent links.
6. Records arrive in MFT order, not tree order, so children often appear before their parents. Collect everything first, then link parents in a second pass.
7. Map NTFS system metafiles (`$MFT`, `$LogFile`, `$Bitmap` and so on) into a single synthetic "NTFS metadata" node under the root.
8. When the user scans a subfolder rather than a whole drive with the MFT method, scan the whole volume and present the requested subtree.

Write our own focused parser instead of a general NTFS crate: we parse a handful of attributes and need large-chunk sequential reads. Existing crates may be used as dev-dependencies to cross-check results in tests.

### Validating the MFT scanner

- Unit tests parse captured raw MFT records stored as test fixtures: resident data, non-resident data, attribute list, multiple hard link names, deleted records.
- An integration test, run manually and elevated, scans the same volume with both methods and asserts that total size and file count agree within a small tolerance. Document why any difference is expected.

---

## 8. Elevation

- The app manifest uses `asInvoker`. The app always starts unelevated.
- `elevation.rs` detects whether the process is elevated.
- "Restart as admin for fast scan" relaunches the app with `ShellExecuteExW` and the `runas` verb, passing the chosen volume as a command line argument. The new instance starts that scan immediately and the old instance exits.

---

## 9. Backend commands and events

All exposed through `tauri-specta`, so TypeScript types are generated. Scans and duplicate searches run on background threads; commands return immediately.

### Volumes and scanning

- `list_volumes() -> Vec<VolumeSummary>` (letter, label, file system, total, free, supports fast scan)
- `start_scan(target: ScanTarget, method: ScanMethod, progress: Channel<ScanProgressUpdate>) -> ScanId`
- `cancel_scan(scan_id)`
- `get_scan_summary(scan_id) -> ScanSummary` (totals, duration, method, inaccessible count, cloud size)

### Tree queries

All tree queries take a `ScanId` plus a `NodeIndex`.

- `get_children(scan_id, node, sort: ChildSortOrder, offset, limit) -> ChildrenPage`
- `get_node_details(scan_id, node) -> NodeDetails` (full path, sizes, counts, dates, flags)
- `get_path_to_root(scan_id, node) -> Vec<BreadcrumbItem>`
- `get_visualization_subtree(scan_id, node, max_depth, minimum_share_of_parent) -> VisualizationNode`
  - Returns a pruned nested structure for the treemap and sunburst.
  - Children below the threshold are merged into one "N smaller items" node.
  - Hard cap the node count around 5,000.
- `get_largest_files(scan_id, node, category_filter, limit) -> Vec<FileSummary>`
- `get_file_type_breakdown(scan_id, node) -> Vec<FileTypeShare>` (by category and by extension)
- `search_nodes(scan_id, query, limit) -> Vec<SearchResult>` (case-insensitive substring match on names; run on a background thread)

### File actions

- `reveal_in_explorer(scan_id, node)`
- `open_with_default_app(scan_id, node)`
- `move_to_recycle_bin(scan_id, nodes) -> RecycleResult`
  - Uses `IFileOperation` with the allow-undo flag.
  - After success, remove the nodes from the tree and re-aggregate the ancestors' sizes, so the UI updates without a rescan.

### Duplicates

- `get_default_duplicate_folders() -> Vec<SuggestedFolder>` (Known Folders: Pictures, Videos, Music, Documents, Downloads, Desktop)
- `start_duplicate_search(settings: DuplicateSearchSettings, progress: Channel<DuplicateSearchProgress>) -> DuplicateSearchId`
- `cancel_duplicate_search(id)`
- `get_duplicate_groups(id, sort, offset, limit) -> DuplicateGroupsPage`
- `get_thumbnail(path, size) -> ThumbnailHandle`
  - Uses the Windows shell thumbnail API (`IShellItemImageFactory`), so videos get frames and documents get previews.
  - Cache PNGs in the app cache directory and return a path the frontend loads through the asset protocol (`convertFileSrc`).
- `recycle_duplicates(id, files_to_remove) -> RecycleResult`

---

## 10. Duplicate finder pipeline

1. **Collect candidates** from the selected folders, keeping only the selected categories' extensions.
   - Ignore files under 64 KB by default (configurable).
   - Skip cloud placeholders and report their count as "N files are online-only and were not checked".
   - Skip reparse points.
2. **Collapse hard links.** Paths sharing a file reference are the same file and never count as duplicates.
3. **Group by exact size.** Drop groups with a single file.
4. **Partial hash.** xxh3 over the first and last 64 KB. Drop groups that split into singles.
5. **Full hash.** BLAKE3 streamed in 1 MB buffers.
6. **Read strategy per drive.** Query the seek penalty (`IOCTL_STORAGE_QUERY_PROPERTY`, `StorageDeviceSeekPenaltyProperty`).
   - SSDs hash files in parallel.
   - HDDs hash sequentially, one file at a time, ordered by path.
7. **Before recycling, revalidate** each file's size and modification time. If anything changed, skip that file and report it.

### Keep rules

Rules apply to every group by default, and the user can override any group by hand.

- Keep the oldest (default)
- Keep the newest
- Keep the copy inside a preferred folder (user picks the folder)
- Keep the shortest path

A group can never end up with zero kept files; enforce this both in the backend and in the UI.

---

## 11. UI and UX

Overall feel: clean, calm, modern, fast. Generous spacing, clear hierarchy, readable numbers. Dark and light themes following the system setting by default.

### Layout

- **Left sidebar** (shadcn Sidebar, collapsible)
  - Drives list, each with a small usage bar.
  - "Scan folder…"
  - "Duplicates"
  - Recent scans in this session.
- **Top bar**
  - Breadcrumbs for the current node.
  - Scan method indicator ("Fast scan" / "Standard scan").
  - Rescan button.
  - Search trigger (`Ctrl+K` opens a shadcn Command palette).
- **Main area**: a resizable split.
  - **Left: tree table** (virtualized). Columns:
    - Name with file type icon
    - Size (with an inline proportional bar)
    - Percentage of parent
    - Files
    - Modified

    Expand and collapse with smooth height animation. Fully keyboard navigable: arrows, Enter to drill in, Backspace to go up.
  - **Right: visualization tabs**: Treemap / Sunburst / File types / Largest files.
- **Inspector panel** (collapsible, right edge or bottom)
  - Details of the selected item and quick actions.
  - Badges for flags (hard link, cloud, compressed, inaccessible).

### Screens and states

- **Home / no scan yet.** Drive cards with used and free space, a donut per drive, and a big "Scan" button per drive. Show "Fast scan available as admin" on NTFS drives when unelevated.
- **Scan in progress.**
  - Live counters for files, folders and size, ticking smoothly.
  - Elapsed time and the current path in muted monospace.
  - A live list of the biggest top-level folders found so far, reordering with layout animation.
  - A cancel button.
- **Results.** The main layout described above.
- **Empty and error states** for every view: access denied, scan cancelled, no results. They need friendly copy and a next action.

### Visualizations

- **Treemap** (custom, `<canvas>`, squarified layout from `d3-hierarchy`)
  - Color by file category (default) or by depth, switchable.
  - Use a consistent palette shared with the file types chart.
  - Hover shows a tooltip and an outline; labels appear only when the rectangle is large enough.
  - Click to zoom into a folder with an animated transition (rectangles interpolate to their new positions, ~300 ms). Breadcrumbs and the tree table stay in sync with the zoom.
  - Right-click opens the same context menu as the tree table.
  - Must stay smooth at the ~5,000 node cap. Render on canvas; hit-test with a precomputed rectangle list.
- **Sunburst** (`d3-hierarchy` partition)
  - Same color system, hover and zoom behavior.
  - Center shows the current folder's size.
- **File types**
  - A shadcn chart donut by category plus a bar chart of the top 15 extensions by size.
  - Clicking a category filters the "Largest files" tab.
- **Largest files**
  - A virtualized list with a category filter, size bars and inline actions.

### Duplicates screen

- **Setup step**
  - Folder checkboxes, pre-filled with known folders and showing each folder's size where cheap, plus "Add folder".
  - Category checkboxes with icons.
  - A minimum size setting.
  - A big "Find duplicates" button.
- **Progress step**
  - The pipeline stages shown as a stepper: Collecting → Comparing sizes → Quick check → Full check.
  - Live counts per stage.
- **Results step**
  - Summary cards: duplicate groups, wasted space, space selected to free.
  - A keep rule selector.
  - A virtualized list of groups sorted by wasted space. Each group shows thumbnails of each copy, the path, the date, and a keep/remove toggle per file.
  - A sticky footer: "Move N files (X GB) to Recycle Bin" plus a confirmation dialog that lists what will be moved.
  - After the action, a toast with the space freed.

### Motion guidelines

Motion should make the app feel responsive and a little fun, never slow.

Use motion for:

- Treemap and sunburst zoom transitions.
- Tree row expand/collapse.
- Layout reordering of the live "biggest folders" list during a scan.
- Number tickers on scan counters and summary cards.
- Tab and panel transitions (short fade or slide).
- Hover and press feedback on cards and buttons.
- Duplicate groups animating out after recycling.

Rules:

- Durations 150–300 ms. Ease-out for entering, ease-in for leaving.
- Define transition presets once in `src/lib/motion/` and reuse them.
- Respect `prefers-reduced-motion`: replace movement with instant changes or simple fades.
- Never animate every row of a long list individually; animate only the first render and the row that changed.
- Never block interaction while an animation runs.

### Frontend state and data flow

- `zustand` stores, one per feature: `useScanStore`, `useExplorerSelectionStore`, `useDuplicateSearchStore`.
- Stores hold IDs and UI state, not big data. Pages of children and visualization subtrees are fetched on demand through hooks in each feature (`useChildrenPage`, `useVisualizationSubtree`), cached by `(scanId, nodeIndex, …)` and invalidated when a recycle action changes the tree.

---

## 12. Performance targets

Measure on a typical SSD system drive with about 1 million files.

| Metric | Target |
|---|---|
| Folder walk | under 60 s |
| MFT scan | under 10 s |
| Memory for the tree | under ~150 bytes per node including names |
| Folder navigation (children query) | under 50 ms |
| Visualization subtree query | under 100 ms |
| Treemap hover and zoom | 60 fps |
| UI during a scan | stays fully responsive |

Add a `cargo bench` or a timed integration test for both scan methods and for tree queries.

---

## 13. Milestones

Each milestone ends with passing `cargo fmt --check`, `cargo clippy`, `cargo test`, `biome ci`, `tsc --noEmit` and `vitest run`.

1. **Foundations**
   - Workspace, Tauri app shell, React + Tailwind + shadcn set up.
   - Biome configured.
   - `tauri-specta` bindings generation wired up.
   - App layout with sidebar and theme switching.
   - `list_volumes` working end to end.
2. **File tree and folder walk**
   - `file_tree` module with tests: aggregation, hard links, node size assertion.
   - Parallel folder walk.
   - Progress channel, cancellation, scan progress screen.
3. **Explorer**
   - Children, details and breadcrumb queries.
   - Virtualized tree table, inspector, context menu.
   - Reveal, open, move to Recycle Bin with tree updates.
4. **Visualizations**
   - Visualization subtree query.
   - Treemap with zoom; sunburst; file types charts; largest files.
   - Command palette search.
5. **MFT scanner**
   - Volume layout, data runs, chunk reader, fixups, attribute parsers, tree linking.
   - Fixture tests and the cross-check against the folder walk.
   - Elevation detection and relaunch flow.
6. **Duplicate finder**
   - Pipeline crate with tests, using temp directories with real duplicates, hard links and size collisions.
   - Thumbnails (images, videos, PDFs and Office documents through the shell thumbnail API, file type icons as fallback), setup, progress and results screens, keep rules, recycle flow.
7. **Polish**
   - Empty and error states, keyboard shortcuts, reduced motion.
   - Performance pass against the targets.
   - Installer configuration (NSIS via Tauri) using the product name and identifier from the Naming table, with code signing hooks left configurable.
   - App icon and wordmark placeholders in `src-tauri/icons/`, ready to swap for final Vexbyte branding.

---

## 14. Testing

### Rust

- Unit tests for every parser and for tree building.
- Integration tests build temporary directory structures containing:
  - nested folders, empty folders
  - hard links (`std::fs::hard_link`)
  - junctions (create with `mklink /J` from the test; skip with a message if that fails)
  - long paths over 260 characters
  - files with identical and different contents of equal size

### Frontend

- Vitest for pure logic: treemap layout preparation, formatting, keep rule selection, color mapping.
- Keep logic out of components so it is testable without rendering.

### Manual checklist per release

- Scan C: with both methods.
- Scan a OneDrive folder and confirm no files get downloaded.
- Scan an exFAT USB drive; it must fall back to the folder walk.
- Run a duplicate search on an external HDD.

---

## 15. Safety rules for development and automated tests

These rules bind every contributor, human or agent, during development.

- **Automated tests never delete, move or modify anything outside a temporary directory they created themselves.** Fixtures live under `std::env::temp_dir()` in a uniquely named folder and are removed by the test that created them.
- **No automated test ever calls the Recycle Bin or any destructive shell operation.** `move_to_recycle_bin` and `recycle_duplicates` are covered only by (a) unit tests of the planning logic (which files would be moved, keep-rule enforcement, revalidation) and (b) manual testing by the maintainer. Tests may assert that the planning layer refuses to empty a duplicate group, but must never execute the operation.
- **Scanning is read-only.** Both scan methods open volumes and directories with read access only. The MFT reader never writes to a volume handle. Cloud placeholder files are never opened.
- **Development builds run against real drives only in read-only paths** (scanning, listing, hashing). Anything that changes the file system is exercised only against temporary fixtures, and only by hand.

---

## 16. Portability rules

Windows and Linux (since 0.2.0) are the build targets, and the app must stay easily portable to macOS. These rules keep platform code at one seam.

### Layout

- Each crate has a `platform/` module instead of `windows_api/`. `platform/mod.rs` does `#[cfg(windows)] mod windows; #[cfg(windows)] pub use windows::*;` and the same with `#[cfg(target_os = "linux")]` for `linux` (macOS would add `macos`). `platform/windows/` holds the Win32 wrappers, `platform/linux/` the Linux code, one responsibility per file.
- No trait, no dynamic dispatch, no runtime selection: each backend exposes the same free functions with the same signatures and the compiler enforces the contract.

### Platform-neutral types at the seam

Functions in `platform/` take and return only crate-owned types, never `windows::` types.

- `DirectoryEntry { name, kind, logical_size, allocated_size, modified_time: SystemTime, identity: FileIdentity, flags: EntryFlags }`
- `FileIdentity` is an opaque newtype (Windows: volume serial + file ID; Linux: device + inode, and only for files with more than one link). Only `Eq`, `Hash`, `Clone`, `Copy` are exposed. The folder walk and the duplicate finder use it for hard-link tracking. The MFT module may keep `FileReference(u64)` internally.
- `EntryFlags` bitflags with neutral names: `Symlink`, `Junction`, `CloudPlaceholder`, `Hidden`, `System`, `Inaccessible`, `Compressed`, `Sparse`. On Linux `Junction` marks a mount point (a directory whose device differs from its parent's), so a scan never leaves the file system it started on.
- `VolumeSummary { mount_point: PathBuf, label, file_system, total, free, supports_fast_scan }`. `mount_point` (for example `C:\`) is the primary identifier, not a drive letter.
- Platform errors are `thiserror` enums with `source: std::io::Error`. On Windows build them with `io::Error::from_raw_os_error` or `.into()` from `windows::core::Error` so the exact Win32 code and message are kept. `windows::core::Error` never appears in a public type outside `platform/windows/`.
- Times cross the seam as `SystemTime`, converted inside the backend. The file tree `Node` may still store a compact `u64` internally.

### Rules

- `windows`, `windows-sys` and `libc` appear only under `[target.'cfg(windows)'.dependencies]` (and later `cfg(unix)`) in Cargo.toml.
- `#[cfg(windows)]` (and any other OS cfg) is allowed only in `platform/mod.rs`, in `crates/mount_table/src/lib.rs` (a Linux-only crate, gated as a whole), in the `mft` module root (the MFT scanner is the one legitimately Windows-only feature, gated as a whole), in `src-tauri/src/elevation.rs`, and in Cargo.toml. Nowhere else. A CI grep enforces this.
- No path handling that assumes backslashes or drive letters outside `platform/`. Use `Path` and `PathBuf` methods. The `\\?\` extended-length prefix is applied inside `platform/windows/` only.
- Use existing cross-platform crates where they already wrap the OS correctly: `trash` for the Recycle Bin (its Windows backend uses `IFileOperation` with undo) and `dirs` for the known folders (Pictures, Videos, Music, Documents, Downloads, Desktop). Reveal in file manager, thumbnails (`IShellItemImageFactory`), SSD seek-penalty detection, cloud-placeholder detection and the MFT scanner stay as Windows implementations inside `platform/windows/`.
- Fast scan is exposed to the UI purely as a capability (`supports_fast_scan` on `VolumeSummary`). The UI already handles "fast scan unavailable" for exFAT or unelevated; other platforms hit the same path.
- CI runs the full check (Biome, tsc, Vitest, fmt, clippy, cargo test) on both `windows-latest` and `ubuntu-22.04`.
- Paths are case-insensitive on Windows and case-sensitive on Linux. Code that compares paths or names asks the platform (`is_same_or_inside_folder`, the frontend's `pathsAreCaseSensitive`); tree lookups try an exact name match before falling back to ignoring case. Search stays case-insensitive everywhere.
- Linux mount facts (mount table, labels, disk space, rotational and removable flags) come from the `mount_table` crate, which reads `/proc/self/mountinfo`, `/sys` and `statvfs` only.
- Known Linux limitation: file names that are not valid UTF-8 are shown with replacement characters and marked inaccessible.

### Feature matrix

| Feature | Windows | macOS (maybe in the future) | Linux (0.2.0) |
|---|---|---|---|
| Folder walk | yes | yes | yes |
| Fast scan | MFT | none | none |
| Elevation | `runas` | not needed | not needed |
| Recycle | `IFileOperation` via `trash` | `trash` | `trash` |
| Reveal | Explorer | `open -R` | D-Bus `FileManager1.ShowItems`, falling back to `xdg-open` on the folder |
| Thumbnails | shell image factory | QuickLook | existing freedesktop thumbnail cache, then own decoder for common images; no icons |
| Cloud placeholders | file attributes | dataless via `getattrlist` | none |
| SSD detection | seek penalty ioctl | IOKit | sysfs `rotational` |
