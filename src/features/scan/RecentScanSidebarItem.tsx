import {
  CircleCheckIcon,
  CircleSlashIcon,
  LoaderCircleIcon,
  TriangleAlertIcon,
  XIcon,
} from "lucide-react";
import { SidebarMenuAction, SidebarMenuButton, SidebarMenuItem } from "@/components/ui/sidebar";
import type { RecentScanDescription } from "./describeRecentScan";

const STATUS_ICONS = {
  running: LoaderCircleIcon,
  finished: CircleCheckIcon,
  cancelled: CircleSlashIcon,
  failed: TriangleAlertIcon,
} as const;

type RecentScanSidebarItemProps = {
  recentScan: RecentScanDescription;
  isActive: boolean;
  onOpen: () => void;
  onDiscard: () => void;
};

export function RecentScanSidebarItem({
  recentScan,
  isActive,
  onOpen,
  onDiscard,
}: RecentScanSidebarItemProps) {
  const StatusIcon = STATUS_ICONS[recentScan.status];

  return (
    <SidebarMenuItem>
      <SidebarMenuButton
        className="h-auto py-1.5"
        isActive={isActive}
        onClick={onOpen}
        tooltip={`${recentScan.location} · ${recentScan.detail}`}
      >
        <StatusIcon
          className={
            recentScan.status === "running" ? "animate-spin motion-reduce:animate-none" : ""
          }
        />
        <div className="grid min-w-0 flex-1">
          <span className="truncate">{recentScan.title}</span>
          <span className="truncate text-sidebar-foreground/60 text-xs">{recentScan.detail}</span>
        </div>
      </SidebarMenuButton>
      <SidebarMenuAction
        showOnHover
        onClick={onDiscard}
        aria-label={`Remove the scan of ${recentScan.title} from this list`}
        title="Remove from list"
      >
        <XIcon />
      </SidebarMenuAction>
    </SidebarMenuItem>
  );
}
