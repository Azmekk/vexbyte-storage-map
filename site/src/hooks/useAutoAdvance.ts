import { useInView, useReducedMotion } from "motion/react";
import { type RefObject, useCallback, useEffect, useState } from "react";

export const AUTO_ADVANCE_MILLISECONDS = 6000;

/**
 * Cycles through `count` items while the element is on screen. Picking an item by hand stops the
 * cycle for good, so the page never takes control back from the visitor.
 */
export function useAutoAdvance(elementRef: RefObject<HTMLElement | null>, count: number) {
  const [activeIndex, setActiveIndex] = useState(0);
  const [isAutomatic, setIsAutomatic] = useState(true);
  const isInView = useInView(elementRef, { amount: 0.4 });
  const prefersReducedMotion = useReducedMotion();
  const isCycling = isAutomatic && isInView && !prefersReducedMotion;

  useEffect(() => {
    if (!isCycling) return;
    const timer = window.setTimeout(
      () => setActiveIndex((index) => (index + 1) % count),
      AUTO_ADVANCE_MILLISECONDS,
    );
    return () => window.clearTimeout(timer);
  }, [isCycling, count]);

  const select = useCallback((index: number) => {
    setIsAutomatic(false);
    setActiveIndex(index);
  }, []);

  // activeIndex is part of the key so the progress bar restarts with each item.
  return { activeIndex, select, isCycling, cycleKey: `${activeIndex}-${isCycling}` };
}
