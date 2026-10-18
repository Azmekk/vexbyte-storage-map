/// <reference types="vitest/config" />
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import react from "@vitejs/plugin-react";
import { defineConfig, searchForWorkspaceRoot } from "vite";

// The app's version is the fallback download when the GitHub API can't be reached.
const appPackage = JSON.parse(
  readFileSync(new URL("../package.json", import.meta.url), "utf8"),
) as {
  version: string;
};

// The changelog page bundles the hand-written CHANGELOG.md from the repository root.
const changelogPath = fileURLToPath(new URL("../CHANGELOG.md", import.meta.url));

// Vite requires a default export for its config file.
export default defineConfig({
  plugins: [react()],
  // GitHub Pages serves a project site from /<repo>/; set SITE_BASE for that host.
  base: process.env.SITE_BASE ?? "/",
  define: {
    __APP_VERSION__: JSON.stringify(appPackage.version),
  },
  server: {
    fs: {
      allow: [searchForWorkspaceRoot(process.cwd()), changelogPath],
    },
  },
  build: {
    target: "es2022",
    // Two pages: the landing page and /changelog/, which the app opens after an update.
    rollupOptions: {
      input: {
        main: fileURLToPath(new URL("index.html", import.meta.url)),
        changelog: fileURLToPath(new URL("changelog/index.html", import.meta.url)),
      },
    },
  },
  test: {
    include: ["src/**/*.test.ts"],
    environment: "node",
  },
});
