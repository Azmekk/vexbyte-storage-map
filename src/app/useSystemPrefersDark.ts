import { useSyncExternalStore } from "react";

const DARK_COLOR_SCHEME_QUERY = "(prefers-color-scheme: dark)";

function subscribeToColorSchemeChanges(onChange: () => void): () => void {
  const mediaQueryList = window.matchMedia(DARK_COLOR_SCHEME_QUERY);
  mediaQueryList.addEventListener("change", onChange);
  return () => mediaQueryList.removeEventListener("change", onChange);
}

function readSystemPrefersDark(): boolean {
  return window.matchMedia(DARK_COLOR_SCHEME_QUERY).matches;
}

/** Tracks the Windows light/dark app mode as reported by WebView2. */
export function useSystemPrefersDark(): boolean {
  return useSyncExternalStore(subscribeToColorSchemeChanges, readSystemPrefersDark);
}
