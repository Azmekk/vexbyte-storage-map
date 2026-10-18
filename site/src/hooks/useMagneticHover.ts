import { animate, useReducedMotion } from "motion/react";
import { type PointerEvent, useCallback } from "react";

const PULL_X = 0.12;
const PULL_Y = 0.25;

/** Lets an element lean toward the pointer and spring back when it leaves. */
export function useMagneticHover() {
  const prefersReducedMotion = useReducedMotion();

  const onPointerMove = useCallback(
    (event: PointerEvent<HTMLElement>) => {
      const element = event.currentTarget;
      const bounds = element.getBoundingClientRect();
      const x = event.clientX - bounds.left;
      const y = event.clientY - bounds.top;
      element.style.setProperty("--mx", `${x}px`);
      element.style.setProperty("--my", `${y}px`);
      if (prefersReducedMotion) return;
      element.style.transform = `translate(${(x - bounds.width / 2) * PULL_X}px, ${(y - bounds.height / 2) * PULL_Y}px)`;
    },
    [prefersReducedMotion],
  );

  const onPointerLeave = useCallback((event: PointerEvent<HTMLElement>) => {
    animate(
      event.currentTarget,
      { transform: "translate(0px, 0px)" },
      { type: "spring", stiffness: 300, damping: 18 },
    );
  }, []);

  return { onPointerMove, onPointerLeave };
}
