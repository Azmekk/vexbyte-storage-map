import { useCallback, useEffect, useMemo, useRef } from "react";
import { easeOutCubic } from "./treemap/treemapViewport";

export type FrameAnimation = {
  /** Calls `onFrame` with eased progress from 0 to 1 over `durationMilliseconds`. */
  run: (durationMilliseconds: number, onFrame: (easedProgress: number) => void) => void;
  cancel: () => void;
};

/** One requestAnimationFrame loop at a time; starting a new run cancels the running one. */
export function useFrameAnimation(): FrameAnimation {
  const frameRequestRef = useRef<number | null>(null);

  const cancel = useCallback(() => {
    if (frameRequestRef.current !== null) {
      cancelAnimationFrame(frameRequestRef.current);
      frameRequestRef.current = null;
    }
  }, []);

  const run = useCallback(
    (durationMilliseconds: number, onFrame: (easedProgress: number) => void) => {
      cancel();
      const startTime = performance.now();

      const step = (now: number) => {
        const progress = Math.min((now - startTime) / durationMilliseconds, 1);
        onFrame(easeOutCubic(progress));
        frameRequestRef.current = progress < 1 ? requestAnimationFrame(step) : null;
      };

      onFrame(0);
      frameRequestRef.current = requestAnimationFrame(step);
    },
    [cancel],
  );

  useEffect(() => cancel, [cancel]);

  return useMemo(() => ({ run, cancel }), [run, cancel]);
}
