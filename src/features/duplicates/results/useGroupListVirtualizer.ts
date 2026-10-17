import { useVirtualizer } from "@tanstack/react-virtual";
import { useEffect, useRef } from "react";
import type { DuplicateGroup } from "@/lib/backend/generated";
import { estimateGroupCardHeight } from "./describeDuplicateGroup";

/** Groups loaded beyond the rendered ones, so scrolling rarely reaches an unloaded page. */
const PREFETCH_GROUP_COUNT = 20;
const OVERSCAN_GROUP_COUNT = 4;

type GroupListVirtualizerOptions = {
  groupCount: number;
  groupAt: (groupIndex: number) => DuplicateGroup | undefined;
  requestGroupRange: (firstIndex: number, lastIndex: number) => void;
};

/** Virtualizes the group list and loads the pages around what is on screen. */
export function useGroupListVirtualizer({
  groupCount,
  groupAt,
  requestGroupRange,
}: GroupListVirtualizerOptions) {
  const scrollElementRef = useRef<HTMLDivElement | null>(null);

  const virtualizer = useVirtualizer({
    count: groupCount,
    getScrollElement: () => scrollElementRef.current,
    estimateSize: (groupIndex) =>
      estimateGroupCardHeight(groupAt(groupIndex)?.files.length ?? null),
    getItemKey: (groupIndex) => groupAt(groupIndex)?.id ?? `unloaded-${groupIndex}`,
    overscan: OVERSCAN_GROUP_COUNT,
  });

  const virtualItems = virtualizer.getVirtualItems();
  const firstVisibleIndex = virtualItems[0]?.index ?? 0;
  const lastVisibleIndex = virtualItems.at(-1)?.index ?? 0;

  useEffect(() => {
    requestGroupRange(
      firstVisibleIndex - PREFETCH_GROUP_COUNT,
      lastVisibleIndex + PREFETCH_GROUP_COUNT,
    );
  }, [firstVisibleIndex, lastVisibleIndex, requestGroupRange]);

  return { scrollElementRef, virtualizer, virtualItems };
}
