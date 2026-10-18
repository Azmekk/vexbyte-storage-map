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
   ARM and 32-bit Windows), and falls back to the user agent string.
2. `fetchLatestRelease` asks the GitHub API for the latest release tag and builds a direct link to
   `Vexbyte.Storage.Map_<version>_x64-setup.exe` (see `installerFileName` in
   `src/lib/release/repository.ts`; change it there if the release workflow ever renames the
   installer). The asset list only supplies the download size. The answer is cached for the browser
   session.
3. If GitHub can't be reached, the button links to the installer for the app version in the root
   `package.json`, which is read at build time. So a site built from a release commit always has a
   working fallback.
4. 64-bit Windows (x64 and ARM) gets the installer directly. macOS and Linux see a "Coming soon
   to …" notice; phones, tablets and 32-bit Windows see "Not available for …". Both come with a
   link to the GitHub releases page and a copy-link button.

The GitHub API allows 60 unauthenticated requests per hour per visitor IP, which a download page
never comes close to.

## Hosting

Nothing is deployed yet. `pnpm build` produces plain static files, so any static host works.

**GitHub Pages** (the simplest fit, since the code and releases already live on GitHub). Build with
`SITE_BASE=/vexbyte-storage-map/ pnpm build` for the project URL
`https://azmekk.github.io/vexbyte-storage-map/`, or leave `SITE_BASE` unset with a custom domain. A
workflow using `actions/upload-pages-artifact` and `actions/deploy-pages` publishes `site/dist`.

**Cloudflare Pages** works the same way: root directory `site`, build command `pnpm build`, output
`dist`. It's only worth choosing if a server-side `/download` redirect or edge caching of the
release lookup is wanted later (Pages Functions).

## Assets

`public/screenshots/` holds copies of the dark screenshots from `docs/screenshots/`. Copy them again
after retaking the screenshots.
