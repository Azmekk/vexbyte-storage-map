import { type RefObject, useLayoutEffect, useState } from "react";
import type { CanvasSize } from "./treemap/treemapViewport";

const EMPTY_SIZE: CanvasSize = { width: 0, height: 0 };

/** The content size of an element in CSS pixels, kept up to date with a ResizeObserver. */
export function useElementSize(elementRef: RefObject<HTMLElement | null>): CanvasSize {
  const [size, setSize] = useState<CanvasSize>(EMPTY_SIZE);

  useLayoutEffect(() => {
    const element = elementRef.current;
    if (element === null) {
      return;
    }

    const updateSize = (width: number, height: number) => {
      const roundedWidth = Math.floor(width);
      const roundedHeight = Math.floor(height);
      setSize((previousSize) =>
        previousSize.width === roundedWidth && previousSize.height === roundedHeight
          ? previousSize
          : { width: roundedWidth, height: roundedHeight },
      );
    };

    const bounds = element.getBoundingClientRect();
    updateSize(bounds.width, bounds.height);

    const observer = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (entry !== undefined) {
        updateSize(entry.contentRect.width, entry.contentRect.height);
      }
    });
    observer.observe(element);
    return () => observer.disconnect();
  }, [elementRef]);

  return size;
}
