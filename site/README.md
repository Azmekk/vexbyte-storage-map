# Download page

The public landing and download page for Vexbyte Storage Map. It's a static Vite + React +
TypeScript site, separate from the app: its own `package.json`, its own dependencies, and no Tauri.

```
pnpm install
pnpm dev          # http://localhost:5173
pnpm build        # static files in dist/
pnpm preview      # serve dist/ locally
pnpm typecheck
pnpm test         # platform detection and release asset picking
```

Biome runs from the repo root and covers this folder too (`pnpm check` at the root).

## How the download button works

1. `detectPlatform` reads User-Agent Client Hints when the browser has them (they reveal Windows on
   ARM, ARM Linux and 32-bit Windows), and falls back to the user agent string.
2. `fetchLatestRelease` asks the GitHub API for the latest release and picks each file from its
   asset list by suffix (GitHub turns the spaces in the product name into dots): `_x64-setup.exe`
   for Windows, and `_amd64.deb`, `.x86_64.rpm` and `_amd64.AppImage` for Linux (see
   `src/lib/release/repository.ts`; change the suffixes there if the release workflow ever renames
   its files). Sizes come from the same list. The answer is cached for the browser session.
3. If GitHub can't be reached, the Windows button links to the installer for the app version in the
   root `package.json`, read at build time, so a site built from a release commit always has a
   working fallback. The Linux file names aren't guessed: without the asset list, Linux visitors see
   "Coming soon to Linux", which is also what they see when the latest release has no Linux files yet.
   That lets the site deploy before a release has finished uploading.
4. 64-bit Windows (x64 and ARM) gets the installer directly; after a click the page explains the
   SmartScreen prompt. x64 Linux gets three buttons (.deb for Debian, Ubuntu and Mint, .rpm for
   Fedora, RHEL and openSUSE, AppImage for Arch and the rest), with one highlighted from the user
   agent, and after a click the install command for that file. macOS sees "Coming soon to macOS",
   ARM Linux an "x64 only for now" notice, and phones, tablets and 32-bit Windows "Not available
   for ...". These come with a link to the GitHub releases page and a copy-link button.

The GitHub API allows 60 unauthenticated requests per hour per visitor IP, which a download page
never comes close to.

## Changelog page

`/changelog/` is a second page (`changelog/index.html`, a second Vite build input), so it is a
plain static file at `dist/changelog/index.html`. It bundles the hand-written `CHANGELOG.md` from
the repository root at build time and parses it with `src/lib/changelog/parseChangelog.ts`. Each
version is a section with `id="<version>"`, which is what the app opens after an update
(`/changelog/#0.2.0`). "What's new" is shown in full and "Developer information" sits in a
collapsed block.

The changelog is written by hand, never generated from commits. Each release adds a section in
this shape, newest first; the release workflow uses the same section for the GitHub release notes
(`scripts/extract-changelog-section.sh`):

```
## 0.2.0 - 2026-10-20
### What's new
- What users notice, in plain words.
### Developer information
- Internals worth knowing about.
```

## Hosting

The site is live at https://storagemap.vexbyte.com/. Cloudflare serves `dist/` as static assets
(`wrangler.jsonc`) and deploys on every push to `main`. A push that changes only `CHANGELOG.md`
redeploys the changelog too, since the build reads it.

`pnpm build` produces plain static files, so any other static host works as well. For a host that
serves the site from a sub-path, set `SITE_BASE` (for example `SITE_BASE=/vexbyte-storage-map/`).

## Assets

`public/screenshots/` holds copies of the dark screenshots from `docs/screenshots/`. Copy them again
after retaking the screenshots.
