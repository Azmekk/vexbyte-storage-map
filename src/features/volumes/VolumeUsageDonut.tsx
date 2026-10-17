import { Pie, PieChart } from "recharts";
import { type ChartConfig, ChartContainer } from "@/components/ui/chart";
import type { VolumeUsageChartSegment } from "./volumeUsage";

const volumeUsageChartConfig = {
  used: { label: "Used", color: "var(--volume-used)" },
  free: { label: "Free", color: "var(--muted)" },
} satisfies ChartConfig;

type VolumeUsageDonutProps = {
  segments: VolumeUsageChartSegment[];
  usedPercentageLabel: string;
};

export function VolumeUsageDonut({ segments, usedPercentageLabel }: VolumeUsageDonutProps) {
  return (
    <div className="relative size-28 shrink-0">
      <ChartContainer config={volumeUsageChartConfig} className="aspect-square size-full">
        <PieChart>
          <Pie
            data={segments}
            dataKey="bytes"
            nameKey="segment"
            innerRadius="68%"
            outerRadius="100%"
            startAngle={90}
            endAngle={-270}
            strokeWidth={0}
            isAnimationActive={false}
          />
        </PieChart>
      </ChartContainer>
      <div className="pointer-events-none absolute inset-0 flex flex-col items-center justify-center">
        <span className="font-semibold text-lg tabular-nums leading-none">
          {usedPercentageLabel}
        </span>
        <span className="mt-1 text-muted-foreground text-xs">used</span>
      </div>
    </div>
  );
}
