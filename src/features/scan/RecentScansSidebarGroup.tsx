import { HistoryIcon } from "lucide-react";
import { useNavigationStore } from "@/app/useNavigationStore";
import {
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarMenu,
} from "@/components/ui/sidebar";
import { RecentScanSidebarItem } from "./RecentScanSidebarItem";
import { discardScanSession } from "./scanSession";
import { useRecentScans } from "./useRecentScans";

export function RecentScansSidebarGroup() {
  const recentScans = useRecentScans();
  const currentView = useNavigationStore((state) => state.currentView);
  const navigateToScan = useNavigationStore((state) => state.navigateToScan);
  const activeScanId = currentView.kind === "scan" ? currentView.scanId : null;

  return (
    <SidebarGroup className="group-data-[collapsible=icon]:hidden">
      <SidebarGroupLabel>Recent scans</SidebarGroupLabel>
      <SidebarGroupContent>
        {recentScans.length === 0 ? (
          <div className="flex items-center gap-2 px-2 py-1 text-sidebar-foreground/60 text-xs">
            <HistoryIcon className="size-3.5" />
            Scans from this session appear here.
          </div>
        ) : (
          <SidebarMenu>
            {recentScans.map((recentScan) => (
              <RecentScanSidebarItem
                key={recentScan.scanId}
                recentScan={recentScan}
                isActive={recentScan.scanId === activeScanId}
                onOpen={() => navigateToScan(recentScan.scanId)}
                onDiscard={() => void discardScanSession(recentScan.scanId)}
              />
            ))}
          </SidebarMenu>
        )}
      </SidebarGroupContent>
    </SidebarGroup>
  );
}
