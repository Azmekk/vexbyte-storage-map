import { motion, useReducedMotion, useSpring, useTransform } from "motion/react";
import { useEffect } from "react";

type AnimatedNumberProps = {
  value: number;
  /** Turns the in-between value into text. Keep it a stable function. */
  format: (value: number) => string;
  className?: string;
};

/** A number that glides to each new value instead of jumping; instant with reduced motion. */
export function AnimatedNumber({ value, format, className }: AnimatedNumberProps) {
  const prefersReducedMotion = useReducedMotion();
  const animatedValue = useSpring(value, { stiffness: 120, damping: 24, restDelta: 0.5 });
  const displayedText = useTransform(animatedValue, (currentValue) => format(currentValue));

  useEffect(() => {
    if (prefersReducedMotion) {
      animatedValue.jump(value);
    } else {
      animatedValue.set(value);
    }
  }, [value, prefersReducedMotion, animatedValue]);

  return <motion.span className={className}>{displayedText}</motion.span>;
}
