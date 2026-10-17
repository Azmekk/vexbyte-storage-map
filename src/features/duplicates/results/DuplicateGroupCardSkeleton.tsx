import { Skeleton } from "@/components/ui/skeleton";

export function DuplicateGroupCardSkeleton() {
  return (
    <div className="overflow-hidden rounded-xl border bg-card" aria-hidden>
      <div className="flex h-[52px] items-center justify-between border-b px-4">
        <Skeleton className="h-4 w-40" />
        <Skeleton className="h-4 w-20" />
      </div>
      <div className="grid divide-y">
        <div className="flex h-16 items-center gap-3 px-3">
          <Skeleton className="size-12 rounded-lg" />
          <Skeleton className="h-4 flex-1" />
        </div>
        <div className="flex h-16 items-center gap-3 px-3">
          <Skeleton className="size-12 rounded-lg" />
          <Skeleton className="h-4 flex-1" />
        </div>
      </div>
    </div>
  );
}
