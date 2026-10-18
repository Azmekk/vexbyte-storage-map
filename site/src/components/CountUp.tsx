import { useRef } from "react";
import { useCountUp } from "../hooks/useCountUp";

type CountUpProps = {
  value: number;
  decimals?: number;
  suffix?: string;
};

export function CountUp({ value, decimals = 0, suffix = "" }: CountUpProps) {
  const elementRef = useRef<HTMLSpanElement>(null);
  const text = useCountUp(elementRef, value, decimals);
  return (
    <span ref={elementRef}>
      {text}
      {suffix}
    </span>
  );
}
