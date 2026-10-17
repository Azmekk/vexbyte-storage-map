import { Progress } from "@/components/ui/progress";
import { SidebarMenuButton, SidebarMenuItem } from "@/components/ui/sidebar";
import { DriveTypeIcon } from "./DriveTypeIcon";
import type { VolumeDescription } from "./describeVolume";

type VolumeSidebarItemProps = {
  volume: VolumeDescription;
  onSelect: () => void;
};

export function VolumeSidebarItem({ volume, onSelect }: VolumeSidebarItemProps) {
  return (
    <SidebarMenuItem>
      <SidebarMenuButton
        className="h-auto py-2"
        onClick={onSelect}
        tooltip={`${volume.displayName} (${volume.driveLabel}) · ${volume.freeSizeText} free`}
      >
        <DriveTypeIcon driveType={volume.driveType} />
        <div className="grid min-w-0 flex-1 gap-1.5">
          <div className="flex items-baseline justify-between gap-2">
            <span className="truncate">
              {volume.displayName} ({volume.driveLabel})
            </span>
            <span className="shrink-0 text-sidebar-foreground/60 text-xs tabular-nums">
              {volume.usedPercentageLabel}
            </span>
          </div>
          <Progress
            value={volume.usedPercentage}
            aria-label={`${volume.usedPercentageLabel} used`}
            className="h-1 *:data-[slot=progress-indicator]:bg-(--volume-used)"
          />
        </div>
      </SidebarMenuButton>
    </SidebarMenuItem>
  );
}
