import { useInView, useReducedMotion } from "motion/react";
import { type RefObject, useEffect, useState } from "react";
import { SEARCH_DEMOS, type SearchDemo } from "../lib/searchDemos";

const ERASE_MILLISECONDS = 18;
const TYPE_MILLISECONDS = 70;
const HOLD_MILLISECONDS = 3200;
const FIRST_DEMO = SEARCH_DEMOS[0] as SearchDemo;

function wait(milliseconds: number, signal: AbortSignal): Promise<void> {
  return new Promise((resolve, reject) => {
    const timer = window.setTimeout(resolve, milliseconds);
    signal.addEventListener("abort", () => {
      window.clearTimeout(timer);
      reject(signal.reason);
    });
  });
}

/** Types example queries into the search card, one after another, while it is on screen. */
export function useSearchDemo(elementRef: RefObject<HTMLElement | null>) {
  const [typedQuery, setTypedQuery] = useState(FIRST_DEMO.query);
  const [demo, setDemo] = useState<SearchDemo>(FIRST_DEMO);
  const isInView = useInView(elementRef, { amount: 0.5 });
  const prefersReducedMotion = useReducedMotion();

  useEffect(() => {
    if (!isInView || prefersReducedMotion) return;
    const controller = new AbortController();
    const { signal } = controller;

    const play = async () => {
      let demoIndex = 1;
      let currentQuery = FIRST_DEMO.query;
      while (!signal.aborted) {
        await wait(HOLD_MILLISECONDS, signal);
        const nextDemo = SEARCH_DEMOS[demoIndex % SEARCH_DEMOS.length] as SearchDemo;
        demoIndex += 1;
        for (let length = currentQuery.length; length >= 0; length -= 1) {
          setTypedQuery(currentQuery.slice(0, length));
          await wait(ERASE_MILLISECONDS, signal);
        }
        for (let length = 1; length <= nextDemo.query.length; length += 1) {
          setTypedQuery(nextDemo.query.slice(0, length));
          await wait(TYPE_MILLISECONDS, signal);
        }
        currentQuery = nextDemo.query;
        setDemo(nextDemo);
      }
    };
    play().catch(() => {
      // Aborted when the card leaves the screen.
    });
    return () => controller.abort();
  }, [isInView, prefersReducedMotion]);

  return { typedQuery, demo };
}
