import { motion, useTransform } from "motion/react";
import { cn } from "@/lib/classNames";
import { useAnimatedNumber } from "./useAnimatedNumber";

type NumberTickerProps = {
  value: number;
  formatValue: (value: number) => string;
  className?: string;
};

/** A number that ticks smoothly to each new value; instant with reduced motion. */
export function NumberTicker({ value, formatValue, className }: NumberTickerProps) {
  const animatedValue = useAnimatedNumber(value);
  const formattedValue = useTransform(animatedValue, formatValue);

  return (
    <motion.span className={cn("tabular-nums", className)} aria-label={formatValue(value)}>
      {formattedValue}
    </motion.span>
  );
}
