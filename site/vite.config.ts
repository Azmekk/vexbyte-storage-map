/// <reference types="vitest/config" />
import { readFileSync } from "node:fs";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// The app's version is the fallback download when the GitHub API can't be reached.
const appPackage = JSON.parse(
  readFileSync(new URL("../package.json", import.meta.url), "utf8"),
) as {
  version: string;
};

// Vite requires a default export for its config file.
export default defineConfig({
  plugins: [react()],
  // GitHub Pages serves a project site from /<repo>/; set SITE_BASE for that host.
  base: process.env.SITE_BASE ?? "/",
  define: {
    __APP_VERSION__: JSON.stringify(appPackage.version),
  },
  build: {
    target: "es2022",
  },
  test: {
    include: ["src/**/*.test.ts"],
    environment: "node",
  },
});
