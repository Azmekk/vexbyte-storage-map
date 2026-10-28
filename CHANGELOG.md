# Changelog

## 0.2.5 - 2026-10-28
### What's new
- Expanding or collapsing a folder no longer scrolls the folder list away when a row further down is selected. The list stays where it is and only scrolls when you select a different item.

## 0.2.4 - 2026-10-28
### What's new
- The fast scan no longer stops with "the MFT is too fragmented to be located" on drives where Windows keeps the MFT's attribute list in its own cluster, which is common on system drives that have been in use for a while. The scan now reads that list from the drive and continues.

## 0.2.3 - 2026-10-22
### What's new
- "Your drives" now shows up-to-date used and free space after you clean up a drive, instead of the numbers from when the app started. It refreshes each time you open the page and when you switch back to the app.

## 0.2.2 - 2026-10-19
### What's new
- The "new version available" prompt is wider, so its buttons fit, and shows a download icon instead of the stars.
- The Recycle Bin confirmations are wider, so long file lists are easier to read.

## 0.2.1 - 2026-10-19
### What's new
- A "Restart as admin" button in the top bar, on every page, restarts the app with admin rights so the fast scan is ready without first picking a drive. It shows on Windows while the app runs without admin rights.

## 0.2.0 - 2026-10-20
### What's new
- Vexbyte Storage Map now runs on Linux. Download a .deb (Debian, Ubuntu, Mint), .rpm (Fedora, RHEL) or AppImage (Arch and others).
- The app now tells you when a new version is out and links to what changed.
### Developer information
- Linux backends behind each crate's `platform/` module, and a new `mount_table` crate that reads `/proc/self/mountinfo`.
- Linux packages are built on Ubuntu 22.04 in the release workflow.

## 0.1.3 - 2026-10-01
### What's new
- First public releases for Windows 10 and 11: disk space explorer, fast scan, duplicate finder.
