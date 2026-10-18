import { useCallback, useState } from "react";

export type CopyState = "idle" | "copied" | "failed";

/** Copies this page's address so a phone or Mac visitor can open it on their PC. */
export function useCopyPageLink() {
  const [copyState, setCopyState] = useState<CopyState>("idle");

  const copyPageLink = useCallback(() => {
    const pageLink = window.location.href.split("#")[0] ?? window.location.href;
    navigator.clipboard
      .writeText(pageLink)
      .then(() => setCopyState("copied"))
      .catch(() => setCopyState("failed"));
  }, []);

  return { copyState, copyPageLink };
}
