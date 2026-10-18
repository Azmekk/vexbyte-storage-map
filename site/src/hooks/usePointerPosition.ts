import { type PointerEvent, useCallback } from "react";

/** Writes the pointer position inside the element to --mx and --my for spotlight effects. */
export function usePointerPosition() {
  return useCallback((event: PointerEvent<HTMLElement>) => {
    const element = event.currentTarget;
    const bounds = element.getBoundingClientRect();
    element.style.setProperty("--mx", `${event.clientX - bounds.left}px`);
    element.style.setProperty("--my", `${event.clientY - bounds.top}px`);
  }, []);
}
