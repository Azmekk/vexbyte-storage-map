import type { ReactNode } from "react";
import { cn } from "@/lib/classNames";
import { NumberTicker } from "../shared/NumberTicker";

type SummaryCardProps = {
  label: string;
  value: number;
  formatValue: (value: number) => string;
  caption: ReactNode;
  isHighlighted?: boolean;
};

export function SummaryCard({
  label,
  value,
  formatValue,
  caption,
  isHighlighted = false,
}: SummaryCardProps) {
  return (
    <div
      className={cn(
        "grid gap-1 rounded-xl border bg-card px-4 py-3.5",
        isHighlighted && "border-(--volume-used)/40 bg-(--volume-used)/8",
      )}
    >
      <p className="text-muted-foreground text-xs">{label}</p>
      <NumberTicker
        value={value}
        formatValue={formatValue}
        className={cn(
          "font-semibold text-3xl tracking-tight",
          isHighlighted && "text-(--volume-used)",
        )}
      />
      <p className="truncate text-muted-foreground text-xs">{caption}</p>
    </div>
  );
}
