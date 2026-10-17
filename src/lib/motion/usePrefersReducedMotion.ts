import { useSyncExternalStore } from "react";

const REDUCED_MOTION_QUERY = "(prefers-reduced-motion: reduce)";

function subscribeToReducedMotionChanges(onChange: () => void): () => void {
  const mediaQueryList = window.matchMedia(REDUCED_MOTION_QUERY);
  mediaQueryList.addEventListener("change", onChange);
  return () => mediaQueryList.removeEventListener("change", onChange);
}

function readPrefersReducedMotion(): boolean {
  return window.matchMedia(REDUCED_MOTION_QUERY).matches;
}

/**
 * The Windows "Animation effects" setting as reported by WebView2. Unlike Motion's
 * `useReducedMotion`, which reads the setting once per mounted component, this follows changes
 * while the app runs.
 */
export function usePrefersReducedMotion(): boolean {
  return useSyncExternalStore(subscribeToReducedMotionChanges, readPrefersReducedMotion);
}
