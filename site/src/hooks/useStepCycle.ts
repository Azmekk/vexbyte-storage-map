import { useInView, useReducedMotion } from "motion/react";
import { type RefObject, useEffect, useState } from "react";

const STEP_MILLISECONDS = 700;
const HOLD_STEPS = 3;

/** Lights steps one by one while the element is on screen, holds, then starts over. */
export function useStepCycle(elementRef: RefObject<HTMLElement | null>, stepCount: number): number {
  const isInView = useInView(elementRef, { amount: 0.6 });
  const prefersReducedMotion = useReducedMotion();
  const [tick, setTick] = useState(0);

  useEffect(() => {
    if (!isInView || prefersReducedMotion) return;
    const timer = window.setInterval(() => setTick((value) => value + 1), STEP_MILLISECONDS);
    return () => window.clearInterval(timer);
  }, [isInView, prefersReducedMotion]);

  if (prefersReducedMotion) return stepCount;
  return Math.min(tick % (stepCount + HOLD_STEPS + 1), stepCount);
}
