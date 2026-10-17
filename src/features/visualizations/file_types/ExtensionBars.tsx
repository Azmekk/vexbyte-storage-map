import { useState } from "react";
import { Bar, BarChart, LabelList, XAxis, YAxis } from "recharts";
import { type ChartConfig, ChartContainer } from "@/components/ui/chart";
import type { FileCategory } from "@/lib/backend/tree";
import { formatBytesAsReadableSize } from "@/lib/formatting/formatBytesAsReadableSize";
import { formatCount } from "@/lib/formatting/formatCount";
import { formatShare } from "@/lib/formatting/formatShare";
import { categoryLabel } from "../categoryPalette";
import type { ExtensionBar } from "./prepareFileTypeCharts";

const barChartConfig = {} satisfies ChartConfig;
const BAR_ROW_HEIGHT = 26;

type ExtensionBarsProps = {
  bars: ExtensionBar[];
  inkColor: string;
  onSelectCategory: (category: FileCategory | null) => void;
};

function formatBarLabel(value: unknown): string {
  return typeof value === "number" ? formatBytesAsReadableSize(value) : "";
}

export function ExtensionBars({ bars, inkColor, onSelectCategory }: ExtensionBarsProps) {
  const [hoveredIndex, setHoveredIndex] = useState<number | null>(null);
  const hoveredBar = hoveredIndex === null ? undefined : bars[hoveredIndex];

  return (
    <div className="grid gap-2">
      <p className="h-4 text-muted-foreground text-xs" aria-live="polite">
        {hoveredBar === undefined
          ? "Hover a bar for details; click it to list the largest files of its kind."
          : `${hoveredBar.label}: ${formatCount(hoveredBar.fileCount)} files, ${formatShare(hoveredBar.share)} of all files · ${categoryLabel(hoveredBar.category)}`}
      </p>
      <ChartContainer
        config={barChartConfig}
        className="aspect-auto w-full"
        style={{ height: bars.length * BAR_ROW_HEIGHT + 8 }}
      >
        <BarChart
          data={bars}
          layout="vertical"
          margin={{ top: 0, right: 72, bottom: 0, left: 0 }}
          barCategoryGap={4}
        >
          <XAxis type="number" dataKey="logicalSize" hide />
          <YAxis
            type="category"
            dataKey="label"
            width={96}
            tickLine={false}
            axisLine={false}
            tick={{ fill: inkColor, fontSize: 12 }}
          />
          <Bar
            dataKey="logicalSize"
            radius={[0, 4, 4, 0]}
            isAnimationActive={false}
            className="cursor-pointer"
            onMouseEnter={(_, barIndex) => setHoveredIndex(barIndex)}
            onMouseLeave={() => setHoveredIndex(null)}
            onClick={(_, barIndex) => {
              const bar = bars[barIndex];
              if (bar !== undefined) {
                onSelectCategory(bar.category);
              }
            }}
          >
            <LabelList
              dataKey="logicalSize"
              position="right"
              formatter={formatBarLabel}
              style={{ fill: inkColor, fontSize: 12 }}
            />
          </Bar>
        </BarChart>
      </ChartContainer>
    </div>
  );
}
