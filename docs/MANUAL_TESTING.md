# Manual test checklist

Run this before tagging a release. Automated tests never touch the Recycle Bin, never elevate and
never scan real drives, so these checks cover what they cannot. Use the **installed build** from
the NSIS installer (or `target/release/VexbyteStorageMap.exe` after `pnpm tauri build`),
not `pnpm tauri dev`: the elevated relaunch only works the way users see it in a built exe.

Each step lists what to do and what you should see. Note anything that differs.

## 0. Prepare a temporary fixture folder

The Recycle Bin checks only ever touch this folder. It lives in your user folder rather than in
`%TEMP%`, because the duplicate finder refuses to search application folders such as `AppData`.
Create it in PowerShell:

```powershell
$fixture = Join-Path $HOME "vsm-manual-test"
New-Item -ItemType Directory -Force "$fixture\Photos", "$fixture\Backup\Photos", "$fixture\Notes" | Out-Null

# Three identical 2 MB "photos" in different folders, plus one that differs in a single byte.
$bytes = [byte[]]::new(2MB); (New-Object Random 42).NextBytes($bytes)
[IO.File]::WriteAllBytes("$fixture\Photos\IMG_0001.jpg", $bytes)
[IO.File]::WriteAllBytes("$fixture\Backup\Photos\IMG_0001.jpg", $bytes)
[IO.File]::WriteAllBytes("$fixture\Backup\Photos\IMG_0001 (copy).jpg", $bytes)
$bytes[1MB] = $bytes[1MB] -bxor 0xFF
[IO.File]::WriteAllBytes("$fixture\Photos\IMG_0001-edited.jpg", $bytes)

# Something to recycle from the explorer.
Set-Content "$fixture\Notes\old-notes.txt" ("lorem ipsum " * 50000)
New-Item -ItemType Directory -Force "$fixture\Notes\Old drafts" | Out-Null
Set-Content "$fixture\Notes\Old drafts\draft.txt" "draft"
$fixture
```

Delete the folder when you are done (`Remove-Item -Recurse $fixture`), after the restore steps
below.

## 1. Home screen

1. Start the app (not as administrator).
   - Every mounted drive has a card with used and free space and a donut.
   - NTFS drives show **Fast scan available as admin**; exFAT and FAT32 drives do not.
   - The sidebar lists the drives with a usage bar, **Scan folder…**, **Duplicates** and
     **Keyboard shortcuts**.
2. Press `?`. The keyboard shortcuts dialog opens; `Esc` closes it.
3. Switch the theme with the moon/sun button: System, Light, Dark. Restart the app; the choice is
   kept.

## 2. Standard scan (folder walk) of C:

1. Click **Scan C:** on the C: card.
   - Files, folders and size counters tick up smoothly; elapsed time and the current path update.
   - The biggest top-level folders reorder live.
   - The window stays responsive: the sidebar collapses, the theme menu opens, other scans can be
     opened from **Recent scans**.
2. Click **Cancel** during a second scan.
   - "Scan cancelled" with **Scan again** and **Back home**. Nothing on disk changed.
3. Let a full scan finish. Write down the elapsed time (target: under 60 s for about a million files
   on an SSD).
   - The explorer opens: tree table on the left, Treemap / Sunburst / File types / Largest files on
     the right, details panel on the right edge.
4. Explore:
   - Expand folders with the chevrons and with `→`/`←`; open one with `Enter` or double-click;
     go up with `Backspace`, `Alt+←`, the breadcrumbs and the **Up** button. Navigation feels
     instant.
   - Treemap: hover shows an outline and a tooltip, click zooms with a short animation, right-click
     shows the same menu as the tree. Toggle **By type** / **By depth**.
   - Sunburst: folders are lightly tinted by the kind of file that fills them (by type); click a
     segment to zoom, click the centre to go up.
   - File types: clicking a category opens Largest files filtered to it.
   - `Ctrl+K` opens search; type part of a name; selecting a result reveals it in the tree.
   - `Ctrl+R` (or `F5`) rescans; the page itself never reloads.
   - `Esc` closes the details panel; the panel button in the top bar opens it again.
5. Open a folder you cannot read, such as `C:\System Volume Information` (shown with an
   "inaccessible" badge).
   - The table explains that Windows did not let the scan look inside, with **Go up**.

## 3. Fast scan (MFT) and the elevated relaunch

1. On the C: card open the split button menu (chevron) and choose **Restart as admin for fast
   scan**.
   - Windows shows the UAC prompt. Click **No**.
   - A dialog says administrator rights were declined and offers a **standard scan** instead. The
     app keeps running.
2. Repeat and click **Yes** in the UAC prompt.
   - The app closes and a new elevated instance opens and immediately starts a **Fast scan** of C:.
   - The scan finishes in seconds (target: under 10 s for about a million files). The top bar badge
     says **Fast scan**.
   - Sizes and file counts of the largest top-level folders match the standard scan closely
     (small differences come from files that changed in between and from NTFS metadata, which the
     fast scan lists as a synthetic item).
3. In the elevated instance, every NTFS card says **Fast scan ready** and its button starts a fast
   scan directly.
4. Close the elevated instance before continuing, so the remaining checks run unelevated.

## 4. OneDrive placeholder safety

Needs a OneDrive folder with **Files On-Demand** and at least one file marked
"Available when online" (cloud icon in File Explorer).

1. Note the cloud-only file's status in File Explorer (cloud icon).
2. **Scan folder…** and pick the OneDrive folder.
   - The scan finishes. Online-only files carry a cloud badge in the details panel and take
     (almost) no space on disk.
3. Open **Duplicates**, add the OneDrive folder, start a search.
   - The progress screen reports "N files are online-only and won't be downloaded or checked".
4. Check File Explorer again: the file still shows the cloud icon, nothing was downloaded, and the
   OneDrive tray icon shows no download activity.

## 4b. Cloud drives

Needs Google Drive for desktop in streaming mode (its drive letter, for example G:). Do not scan
or search the drive any other way while testing.

1. Open the home screen.
   - The Google Drive card shows a **Cloud drive** badge, no fast scan badge and no Scan button;
     instead it reads "Streamed cloud drives aren't scanned, so nothing gets downloaded."
   - In the sidebar the drive has a cloud icon and is greyed out; clicking it does nothing.
2. **Scan folder…** and pick a folder on the Google Drive.
   - A "Cloud drives aren't scanned" toast appears and no scan screen opens.
3. Open **Duplicates**, click **Add folder** and pick a folder on the Google Drive.
   - The folder is not added; the setup step shows the cloud drive message inline.
4. The Google Drive tray icon shows no download activity.

## 5. exFAT fallback

Needs an exFAT (or FAT32) USB drive.

1. Plug it in and click **Look again** if the home screen shows no card for it yet.
   - Its card shows the file system as exFAT and no fast scan badge; the button is a plain
     **Scan X:**.
2. Scan it. The standard scan runs and finishes normally.
3. In an elevated instance (section 3), the exFAT card still offers only the standard scan.

## 6. Duplicate search on an external hard disk

Needs a spinning USB hard disk with some photos or videos on it.

1. **Duplicates** → **Add folder** → pick a folder on the HDD. Untick the personal folders to keep
   it short.
2. **Find duplicates**.
   - The stepper goes Collecting → Comparing sizes → Quick check → Full check with live counts, and
     "Duplicate groups so far" grows during the checks.
   - The disk is read one file at a time (steady, quiet seeking); the app stays responsive.
3. **Cancel** once mid-search: "Search cancelled", nothing changed, **Search again** works.
4. Let it finish: results list groups sorted by wasted space, with thumbnails.

## 7. Recycle Bin from the explorer (fixture folder only)

1. **Scan folder…** → the fixture folder from step 0.
2. Select `Notes\old-notes.txt` in the tree and press `Delete`.
   - A confirmation dialog lists exactly that file and its size. Nothing has moved yet.
   - Click **Cancel**: the file is still there.
3. Press `Delete` again and confirm **Move to Recycle Bin**.
   - A toast reports the space freed; the file disappears from the tree, the treemap and the
     totals of every parent folder.
4. Right-click `Notes\Old drafts` → **Move to Recycle Bin…** → confirm. The folder and its file go.
5. The scanned root itself cannot be recycled: its menu item is disabled.
6. Restore: open the Windows **Recycle Bin**, select `old-notes.txt` and `Old drafts`, right-click
   → **Restore**. Both are back in `Notes`. Rescan the fixture (`Ctrl+R`): they show up again.

## 8. Recycle Bin from the duplicates screen (fixture folder only)

1. **Duplicates** → untick every personal folder → **Add folder** → the fixture folder.
   Keep every kind of file selected and the default minimum size (64 KB). **Find duplicates**.
   - One group: three copies of `IMG_0001.jpg` (2 MB each). `IMG_0001-edited.jpg` is not in it.
   - The oldest copy is marked **Keep**, the other two **Remove**.
2. Try to mark the last kept copy as **Remove**: a toast explains that every group keeps a copy.
3. Choose **Keep the one in a folder…** and pick `Backup\Photos`.
   - A copy from that folder becomes the kept one. The selector shows the folder with a
     **Change** button; switching to another rule and back does not ask for the folder again.
4. Click **Move 2 files (4 MB) to Recycle Bin**.
   - The dialog lists both files with their paths. Click **Cancel**: nothing moved.
5. Click it again and confirm.
   - A toast reports 4 MB freed; the group animates out; "No duplicates left" appears.
6. Files that change after the search are protected: restore the copies (step 7), run the search
   again, then change one of the files marked **Remove** before confirming the move, for example
   `Add-Content "$fixture\Backup\Photos\IMG_0001 (copy).jpg" "x"`.
   - That file is skipped and listed as changed since the search; the others move.
7. Restore from the Recycle Bin as in section 7 and check the copies are back in place.

## 8b. Search

The search index is read-only. Opening or showing a result does not change anything on disk.

1. Start the app (once unelevated, once elevated) and look at **Search files** in the sidebar
   right away.
   - It shows "Warming up…" with a small spinner while the drives are read for the first time,
     and nothing once every local drive is ready. Elevated, the drives use the fast MFT read.
2. Press `Ctrl+Shift+F` (or `F3`) on any screen, including the explorer of a scan.
   - The Search screen opens with the focus in the search box. `Ctrl+K` inside the explorer still
     opens the search of that scan only.
   - Before typing, the screen shows example queries and the **Search index** card: every drive
     with its state, item count, "built … ago", method (Fast or Standard) and estimated memory,
     plus the total memory of the index.
3. While a drive is still warming, type `*.dll`.
   - The "Warming up search engine" banner lists each drive still being read with a ticking file
     count, the elapsed time and Fast or Standard. Results from the ready drives appear at once,
     and the banner says for example "C: still warming, results will update".
   - When the drive finishes, the results refresh by themselves and the banner goes away.
4. Try these queries; each answers within a fraction of a second once the index is ready:
   - `*.dll`: thousands of matches, "showing the first 10,000" when more matched.
   - `ext:png size:>1mb`: only PNG files larger than 1 MB (check the Size column).
   - `type:folder node_modules`: only folders, with a folder icon.
   - `windows\system32 ext:exe`: only files whose path contains `windows\system32`.
   - `size:big`: an inline message under the box says it is not a size; no toast appears.
   - `dm:today`: "The dm: filter is not supported yet."
5. Change **Sort by** to Size, Modified and Path, or click the column headers: the order changes
   and the list jumps back to the top.
6. Press `↓` a few times, `Page Down`, `Ctrl+End`, `Ctrl+Home`: the selection moves and stays in
   view. `Ctrl+C` copies the selected path (a toast confirms). `Esc` clears the query.
7. Right-click a result: **Open**, **Show in File Explorer**, **Copy path** and **Copy name**.
   - **Show in File Explorer** opens an Explorer window with the item selected.
   - On a harmless file (a `.txt` you created), **Open** or double-click or `Enter` opens it with
     its default app. On a folder, it opens the folder in Explorer.
8. In the **Search index** card, click the refresh button of the smallest drive.
   - It switches to "Refreshing" with a ticking count, the old index keeps answering, and when it
     is done "built just now" shows. **Refresh all** re-reads every drive.
9. Cloud, network and removable drives appear as "Not indexed" with the reason; Google Drive for
   desktop shows no download activity.

## 9. Polish

1. Turn on **Settings → Accessibility → Visual effects → Animation effects: Off** and restart the
   app.
   - Treemap and sunburst zooms jump straight to the new folder, counters jump to their values,
     dialogs and menus appear without sliding. Loading spinners still turn.
2. Resize the window to its minimum size (960 × 600): nothing overlaps, the split panels resize.
3. Collapse the sidebar: icons with tooltips remain.

## 10. Installer

1. Run `Vexbyte Storage Map_<version>_x64-setup.exe`.
   - Installs per user without an admin prompt, adds a Start menu entry "Vexbyte Storage Map",
     the app icon is the treemap glyph.
2. Settings → Apps → Installed apps: **Vexbyte Storage Map**, publisher Vexbyte, the right version.
   Uninstall works and removes the Start menu entry.
