import type { DuplicateGroup, DuplicateGroupId } from "@/lib/backend/generated";
import { DuplicateGroupCard } from "./DuplicateGroupCard";
import { DuplicateGroupCardSkeleton } from "./DuplicateGroupCardSkeleton";
import { useGroupListVirtualizer } from "./useGroupListVirtualizer";

type DuplicateGroupListProps = {
  groupCount: number;
  groupAt: (groupIndex: number) => DuplicateGroup | undefined;
  requestGroupRange: (firstIndex: number, lastIndex: number) => void;
  nowUnixMilliseconds: number;
  leavingGroupIds: ReadonlySet<DuplicateGroupId>;
  pathsBeingSaved: ReadonlySet<string>;
  onToggleKeep: (group: DuplicateGroup, path: string) => void;
  onReveal: (path: string) => void;
  onOpen: (path: string) => void;
};

export function DuplicateGroupList({
  groupCount,
  groupAt,
  requestGroupRange,
  nowUnixMilliseconds,
  leavingGroupIds,
  pathsBeingSaved,
  onToggleKeep,
  onReveal,
  onOpen,
}: DuplicateGroupListProps) {
  const { scrollElementRef, virtualizer, virtualItems } = useGroupListVirtualizer({
    groupCount,
    groupAt,
    requestGroupRange,
  });

  return (
    <div ref={scrollElementRef} className="min-h-0 flex-1 overflow-y-auto">
      <div className="mx-auto w-full max-w-6xl px-6 pt-4 pb-6">
        <div
          className="relative w-full"
          style={{ height: virtualizer.getTotalSize() }}
          aria-label="Duplicate groups"
          role="feed"
          aria-busy={virtualItems.some((virtualItem) => groupAt(virtualItem.index) === undefined)}
        >
          {virtualItems.map((virtualItem) => {
            const group = groupAt(virtualItem.index);

            return (
              <div
                key={virtualItem.key}
                ref={virtualizer.measureElement}
                data-index={virtualItem.index}
                className="absolute top-0 left-0 w-full pb-3"
                style={{
                  transform: `translateY(${virtualItem.start}px)`,
                }}
              >
                {group === undefined ? (
                  <DuplicateGroupCardSkeleton />
                ) : (
                  <DuplicateGroupCard
                    group={group}
                    nowUnixMilliseconds={nowUnixMilliseconds}
                    isLeaving={leavingGroupIds.has(group.id)}
                    pathsBeingSaved={pathsBeingSaved}
                    onToggleKeep={onToggleKeep}
                    onReveal={onReveal}
                    onOpen={onOpen}
                  />
                )}
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
