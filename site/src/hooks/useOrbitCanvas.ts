import { useReducedMotion } from "motion/react";
import { type RefObject, useEffect } from "react";
import { drawOrbit } from "../lib/orbit/drawOrbit";

type OrbitOptions = {
  scale: number;
  alpha: number;
  followPointer: boolean;
};

const MAXIMUM_PIXEL_RATIO = 2;
const POINTER_EASING = 0.05;

export function useOrbitCanvas(
  canvasRef: RefObject<HTMLCanvasElement | null>,
  { scale, alpha, followPointer }: OrbitOptions,
): void {
  const prefersReducedMotion = useReducedMotion();

  useEffect(() => {
    const canvas = canvasRef.current;
    const context = canvas?.getContext("2d");
    if (!canvas || !context) return;

    let width = 0;
    let height = 0;
    let pointerX = 0.5;
    let pointerY = 0.5;
    let easedX = 0.5;
    let easedY = 0.5;
    let isVisible = true;
    let frameRequest = 0;
    const startedAt = performance.now();

    const resize = () => {
      const ratio = Math.min(window.devicePixelRatio || 1, MAXIMUM_PIXEL_RATIO);
      width = canvas.clientWidth;
      height = canvas.clientHeight;
      canvas.width = Math.round(width * ratio);
      canvas.height = Math.round(height * ratio);
      context.setTransform(ratio, 0, 0, ratio, 0, 0);
    };

    const render = (now: number) => {
      easedX += (pointerX - easedX) * POINTER_EASING;
      easedY += (pointerY - easedY) * POINTER_EASING;
      drawOrbit(context, {
        width,
        height,
        // With reduced motion the rings appear fully grown and hold still.
        elapsedSeconds: prefersReducedMotion ? 99 : (now - startedAt) / 1000,
        pointerX: easedX,
        pointerY: easedY,
        scale,
        alpha,
      });
    };

    const loop = (now: number) => {
      if (isVisible) render(now);
      frameRequest = requestAnimationFrame(loop);
    };

    const onPointerMove = (event: PointerEvent) => {
      const bounds = canvas.getBoundingClientRect();
      pointerX = (event.clientX - bounds.left) / bounds.width;
      pointerY = (event.clientY - bounds.top) / bounds.height;
    };

    resize();
    const resizeObserver = new ResizeObserver(() => {
      resize();
      if (prefersReducedMotion) render(performance.now());
    });
    resizeObserver.observe(canvas);
    const visibilityObserver = new IntersectionObserver(([entry]) => {
      isVisible = entry?.isIntersecting ?? true;
    });
    visibilityObserver.observe(canvas);

    if (prefersReducedMotion) {
      render(performance.now());
    } else {
      frameRequest = requestAnimationFrame(loop);
      if (followPointer) window.addEventListener("pointermove", onPointerMove);
    }

    return () => {
      cancelAnimationFrame(frameRequest);
      resizeObserver.disconnect();
      visibilityObserver.disconnect();
      window.removeEventListener("pointermove", onPointerMove);
    };
  }, [canvasRef, scale, alpha, followPointer, prefersReducedMotion]);
}
