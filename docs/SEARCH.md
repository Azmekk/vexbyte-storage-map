# Global file search

Vexbyte Storage Map can search every local drive by name or path, in the spirit of
[Everything](https://www.voidtools.com/) by voidtools. The search index is separate from the scans
the user starts: it is built automatically when the app starts and stays in memory for the app's
lifetime.

## Query syntax

The syntax is a subset of Everything's.

| Query | Finds |
|---|---|
| `report` | Names that contain `report`, ignoring case. |
| `annual report 2024` | Names that contain all three terms, in any order (terms are joined by AND). |
| `"annual report"` | Names that contain the phrase, space included. |
| `*.jpg`, `img_????.png` | Wildcards: `*` is any run of characters, `?` exactly one. A term with wildcards must match the **whole** name. |
| `photos\2024`, `users/me/` | A term with `\` or `/` is matched against the **full path** instead of the name. `/` and `\` are the same. |
| `c:\*\*.iso` | Wildcards in a path term match the whole path. |
| `ext:jpg;png;heic` | Files with one of these extensions (`;` or `,` separates them; a leading dot is fine). Multi-part extensions such as `ext:tar.gz` work too. |
| `size:>100mb`, `size:<1kb`, `size:>=1gb`, `size:<=10kb` | Logical size compared with a value. For folders it is the total size of everything inside. |
| `size:1mb..10mb` | An inclusive size range. |
| `size:4096`, `size:=1.5kb` | An exact size. |
| `type:file`, `type:folder` | Only files or only folders (`dir`, `directory` also work). |
| `file:`, `folder:` | Shorthand for `type:file` and `type:folder`; `folder:photos` also requires the term `photos`. |

- Size units are binary, as in Explorer and Everything: `b`, `kb`, `mb`, `gb`, `tb` (also `k`, `m`,
  `g`, `t` and `kib`, `mib`, ...). A number without a unit is bytes. Decimals are allowed.
- Filter names are case-insensitive. A quoted prefix such as `"ext:jpg"` is plain text.
- Filters combine with terms and with each other by AND: `holiday ext:jpg size:>2mb`.
- A filter without a value (`ext:` while still typing) restricts nothing yet.
- An empty query returns no results (not every file).
- Unknown `name:` prefixes such as `note:2024` are searched as text.
- Everything filters that are not supported yet are refused with an `unsupportedFilter` error
  instead of silently matching nothing: `dm:`, `dc:`, `da:`, `datemodified:`, `attrib:`,
  `parent:`, `path:`, `regex:`, `content:`, `case:`, `ww:`, `len:`, `depth:`, `dupe:` and others.
- Not supported: OR (`|`), NOT (`!`), grouping with `< >`, regular expressions, date filters.

## Results

`searchFiles(query, sort, limit)` (in `src/lib/backend/search.ts`) returns a `SearchResponse`:

- `hits`: the best `limit` matches (clamped to 1..10,000) in the requested `SearchSortOrder`:
  `nameAscending`, `sizeDescending`, `modifiedDescending` (unknown times last) or
  `pathAscending`. Every match is considered, so the order is exact, not only the first matches
  found.
- `totalMatchCount` and `totalMatchesCapped`: how many items matched in total, and whether that is
  more than `hits` holds.
- `searchedVolumes`: the drives whose index answered.
- `skippedVolumesStillWarming`: local drives whose first index is not ready yet, so their files are
  missing from the results.
- `elapsedMilliseconds`.

Each `SearchHit` has `volumeMountPoint`, `path`, `name`, `isDirectory`, `logicalSize`,
`modifiedAtUnixMilliseconds`, `category` and `node`. `revealSearchHit(path)` and
`openSearchHit(path)` accept only paths that are in a ready index. The path is compared ignoring
case, as NTFS does, so a path that differs from the hit's only in case is accepted too. There is no Recycle Bin action
from search results in this version; to act on a folder, start a normal scan of it.

Invalid queries reject with a `BackendCommandError` whose `commandError` is
`{ kind: "invalidQuery", problem }`; its message (or `describeSearchQueryError(problem)`) can be
shown under the search box.

## Index status

`getSearchIndexStatus()` returns, and the `search-index-status-changed` event (through
`subscribeToSearchIndexStatus`) pushes, a full `SearchIndexStatus`:

- `overall`:
  - `warming`: the drives are still being listed, or a local drive is still being read for the
    first time. Show "Warming up search engine" with per-drive progress. Drives that are ready can
    already be searched.
  - `ready`: every local drive can be searched (some may be refreshing in the background).
  - `partiallyReady`: every local drive was tried, some failed.
  - `failed`: nothing can be searched (every build failed, or there is no local drive).
- `volumes`: one `VolumeIndexStatus` per drive, ordered by mount point, with `mountPoint`,
  `label`, `volumeKind`, `state` and `index`.
  - `state` is what is happening to the drive's index: `idle` (waiting for a build slot),
    `building` (`filesSoFar`, `directoriesSoFar`, `elapsedMilliseconds`, `method`), `ready`,
    `failed` (`reason`) or `skipped` (`reason`: `cloudDrive`, `networkDrive`, `removableDrive`,
    `opticalDrive`, `ramDisk`, `unknownDriveType`).
  - `index` is set whenever the drive can be searched: `builtAtUnixMilliseconds`, `fileCount`,
    `directoryCount`, `method`, `durationMilliseconds`, `estimatedMemoryBytes`. It stays set while
    a refresh runs and after a refresh failed, because the old index keeps answering.

The event fires on every state change and about four times per second per drive while one is read.

`refreshSearchIndex(mountPoint?)` rebuilds one drive, or every local drive (re-listing the drives,
so newly connected ones are picked up). The old index keeps answering until the new one replaces it.

## How indexing works

- Only `localDisk` volumes are indexed. Cloud drives (Google Drive for desktop and other user-mode
  file systems) are never read, because reading them downloads files. Network, removable, optical
  and RAM drives are skipped as well.
- At most two drives are read at the same time; the system drive goes first.
- The fast NTFS Master File Table read is used when the app runs as administrator; otherwise the
  folder walk. When the MFT read fails, the folder walk is tried.
- Reading is strictly read-only, exactly like a normal scan.
- Memory: the index does not keep the scan's file tree. It keeps a 32-byte record per file or
  folder (parent, extension, size, modification time, depth and a few flags) and the names, so
  it needs roughly 50 to 60 bytes per item: a drive with 3.5 million items needs about 200 MB.
  Names are lowercased while a query runs (most names are already lowercase or ASCII, which
  costs next to nothing), and results are put in path order by walking up the tree, so no
  lowercase copy of the names or path order table is stored.

## Not in this version

- **Live updates.** The index is a snapshot from app start (or the last refresh). Files created,
  renamed or deleted afterwards are not reflected until a refresh. The design is ready for NTFS USN
  change journal updates: each volume's index is an immutable value that is replaced as a whole, so
  a journal reader can produce updated indexes and swap them in the same way.
- Turning automatic indexing off, discarding indexes to free memory, and persisting the index to
  disk between runs.
