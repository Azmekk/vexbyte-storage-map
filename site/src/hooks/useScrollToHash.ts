import { useEffect } from "react";

/**
 * The page renders after load, so the browser's own jump to /changelog/#<version> finds nothing.
 * This repeats the jump once the sections exist.
 */
export function useScrollToHash() {
  useEffect(() => {
    const id = decodeURIComponent(window.location.hash.slice(1));
    if (id === "") return;
    document.getElementById(id)?.scrollIntoView({ behavior: "instant", block: "start" });
  }, []);
}
