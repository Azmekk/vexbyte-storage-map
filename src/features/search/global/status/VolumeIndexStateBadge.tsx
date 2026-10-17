import { LoaderCircleIcon } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/classNames";
import type { VolumeIndexStateDescription } from "./describeVolumeIndexState";

type VolumeIndexStateBadgeProps = {
  description: VolumeIndexStateDescription;
};

export function VolumeIndexStateBadge({ description }: VolumeIndexStateBadgeProps) {
  const { tone, label } = description;

  return (
    <Badge
      variant={tone === "failed" ? "destructive" : tone === "ready" ? "secondary" : "outline"}
      className={cn(tone === "skipped" && "text-muted-foreground")}
    >
      {tone === "busy" && (
        <LoaderCircleIcon aria-hidden className="animate-spin text-(--volume-used)" />
      )}
      {tone === "ready" && (
        <span aria-hidden className="size-1.5 rounded-full bg-emerald-500 dark:bg-emerald-400" />
      )}
      {label}
    </Badge>
  );
}
