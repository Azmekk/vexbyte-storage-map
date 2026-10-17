import { cn } from "@/lib/classNames";

type SizeBarProps = {
  /** 0 to 1. */
  share: number;
  className?: string;
  /** CSS colour of the filled part; the app accent by default. */
  color?: string;
};

/** A thin proportional bar with rounded ends; tiny non-zero shares stay visible. */
export function SizeBar({ share, className, color }: SizeBarProps) {
  const clampedShare = Math.min(Math.max(share, 0), 1);
  const visibleShare = clampedShare > 0 ? Math.max(clampedShare, 0.03) : 0;

  return (
    <div aria-hidden className={cn("h-1.5 overflow-hidden rounded-full bg-muted", className)}>
      <div
        className="h-full rounded-full bg-(--volume-used)"
        style={{
          width: `${visibleShare * 100}%`,
          backgroundColor: color,
        }}
      />
    </div>
  );
}
