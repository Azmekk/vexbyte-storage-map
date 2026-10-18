import { animate, useInView, useReducedMotion } from "motion/react";
import { type RefObject, useEffect, useState } from "react";

export function formatCount(value: number, decimals: number): string {
  return value.toLocaleString("en-US", {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  });
}

/** Counts from zero to the target the first time the element scrolls into view. */
export function useCountUp(
  elementRef: RefObject<HTMLElement | null>,
  target: number,
  decimals: number,
): string {
  const isInView = useInView(elementRef, { once: true, amount: 0.5 });
  const prefersReducedMotion = useReducedMotion();
  const [value, setValue] = useState(target);

  useEffect(() => {
    if (!isInView || prefersReducedMotion) return;
    const controls = animate(0, target, {
      duration: 1.6,
      ease: [0.16, 1, 0.3, 1],
      onUpdate: setValue,
    });
    return () => controls.stop();
  }, [isInView, prefersReducedMotion, target]);

  return formatCount(value, decimals);
}
