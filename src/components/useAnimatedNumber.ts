import { type MotionValue, useSpring } from "motion/react";
import { useEffect } from "react";
import { usePrefersReducedMotion } from "@/lib/motion/usePrefersReducedMotion";

/** A spring that eases to `targetValue` (~250 ms); jumps straight there with reduced motion. */
export function useAnimatedNumber(targetValue: number): MotionValue<number> {
  const prefersReducedMotion = usePrefersReducedMotion();
  const animatedValue = useSpring(targetValue, { stiffness: 140, damping: 26, mass: 0.6 });

  useEffect(() => {
    if (prefersReducedMotion) {
      animatedValue.jump(targetValue);
    } else {
      animatedValue.set(targetValue);
    }
  }, [animatedValue, prefersReducedMotion, targetValue]);

  return animatedValue;
}
