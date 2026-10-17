import { useState } from "react";
import { Pie, PieChart } from "recharts";
import { type ChartConfig, ChartContainer } from "@/components/ui/chart";
import type { FileCategory } from "@/lib/backend/tree";
import { cn } from "@/lib/classNames";
import { formatBytesAsReadableSize } from "@/lib/formatting/formatBytesAsReadableSize";
import { formatCount } from "@/lib/formatting/formatCount";
import { formatShare } from "@/lib/formatting/formatShare";
import type { CategorySlice } from "./prepareFileTypeCharts";

const donutChartConfig = {} satisfies ChartConfig;

type CategoryDonutProps = {
  slices: CategorySlice[];
  totalSize: number;
  surfaceColor: string;
  onSelectCategory: (category: FileCategory | null) => void;
};

export function CategoryDonut({
  slices,
  totalSize,
  surfaceColor,
  onSelectCategory,
}: CategoryDonutProps) {
  const [hoveredKey, setHoveredKey] = useState<string | null>(null);
  const hoveredSlice = slices.find((slice) => slice.key === hoveredKey) ?? null;

  return (
    <div className="flex flex-wrap items-center gap-6">
      <div className="relative size-44 shrink-0">
        <ChartContainer config={donutChartConfig} className="aspect-square size-full">
          <PieChart>
            <Pie
              data={slices}
              dataKey="logicalSize"
              nameKey="label"
              innerRadius="64%"
              outerRadius="100%"
              startAngle={90}
              endAngle={-270}
              stroke={surfaceColor}
              strokeWidth={2}
              isAnimationActive={false}
              className="cursor-pointer outline-none"
              onMouseEnter={(_, sliceIndex) => setHoveredKey(slices[sliceIndex]?.key ?? null)}
              onMouseLeave={() => setHoveredKey(null)}
              onClick={(_, sliceIndex) => {
                const slice = slices[sliceIndex];
                if (slice !== undefined) {
                  onSelectCategory(slice.category);
                }
              }}
            />
          </PieChart>
        </ChartContainer>
        <div className="pointer-events-none absolute inset-0 flex flex-col items-center justify-center px-8 text-center">
          <span className="font-semibold text-lg tabular-nums leading-tight">
            {formatBytesAsReadableSize(hoveredSlice?.logicalSize ?? totalSize)}
          </span>
          <span className="line-clamp-2 text-muted-foreground text-xs">
            {hoveredSlice === null ? "in files" : hoveredSlice.label}
          </span>
        </div>
      </div>

      <ul className="grid min-w-56 flex-1 gap-0.5">
        {slices.map((slice) => (
          <li key={slice.key}>
            <button
              type="button"
              className={cn(
                "grid w-full grid-cols-[auto_minmax(0,1fr)_auto_3.5rem] items-center gap-2 rounded-md px-2 py-1 text-left text-sm outline-none hover:bg-muted/70 focus-visible:ring-2 focus-visible:ring-ring",
                hoveredKey === slice.key && "bg-muted/70",
              )}
              onMouseEnter={() => setHoveredKey(slice.key)}
              onMouseLeave={() => setHoveredKey(null)}
              onClick={() => onSelectCategory(slice.category)}
              title={`${formatCount(slice.fileCount)} files. Show the largest ones.`}
            >
              <span
                aria-hidden
                className="size-2.5 rounded-sm"
                style={{ backgroundColor: slice.fill }}
              />
              <span className="truncate">{slice.label}</span>
              <span className="tabular-nums">{formatBytesAsReadableSize(slice.logicalSize)}</span>
              <span className="text-right text-muted-foreground tabular-nums">
                {formatShare(slice.share)}
              </span>
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}
