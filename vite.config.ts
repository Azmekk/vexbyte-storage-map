/// <reference types="vitest/config" />
import { fileURLToPath } from "node:url";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

const tauriDevelopmentHost = process.env.TAURI_DEV_HOST;

// Vite requires a default export for its config file.
export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: tauriDevelopmentHost ?? false,
    hmr: tauriDevelopmentHost
      ? { protocol: "ws", host: tauriDevelopmentHost, port: 1421 }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**", "**/crates/**", "**/target/**"],
    },
  },
  envPrefix: ["VITE_", "TAURI_ENV_*"],
  build: {
    target: "chrome120",
    sourcemap: Boolean(process.env.TAURI_ENV_DEBUG),
    // The bundle loads from disk inside the app, so one larger chunk costs nothing noticeable.
    chunkSizeWarningLimit: 1500,
  },
  test: {
    include: ["src/**/*.test.ts"],
    environment: "node",
  },
});
