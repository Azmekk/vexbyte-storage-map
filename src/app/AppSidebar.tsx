import { CopyIcon, FolderSearchIcon, HistoryIcon } from "lucide-react";
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuBadge,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSkeleton,
  SidebarRail,
} from "@/components/ui/sidebar";
import type { VolumesState } from "@/features/volumes/useVolumes";
import { VolumeSidebarItem } from "@/features/volumes/VolumeSidebarItem";

const SKELETON_ITEM_KEYS = ["first", "second"];

type AppSidebarProps = {
  volumesState: VolumesState;
};

export function AppSidebar({ volumesState }: AppSidebarProps) {
  return (
    <Sidebar collapsible="icon">
      <SidebarHeader>
        <div className="flex items-center gap-2 px-1 py-1.5">
          <img src="/favicon.png" alt="" className="size-7 shrink-0 rounded-md" />
          <span className="truncate font-semibold group-data-[collapsible=icon]:hidden">
            Vexbyte Storage Map
          </span>
        </div>
      </SidebarHeader>

      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>Drives</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {volumesState.status === "loading" &&
                SKELETON_ITEM_KEYS.map((skeletonKey) => (
                  <SidebarMenuItem key={skeletonKey}>
                    <SidebarMenuSkeleton showIcon />
                  </SidebarMenuItem>
                ))}
              {volumesState.status === "ready" &&
                volumesState.volumes.map((volume) => (
                  <VolumeSidebarItem key={volume.driveLetter} volume={volume} />
                ))}
              {volumesState.status === "error" && (
                <p className="px-2 text-sidebar-foreground/60 text-xs">Drives unavailable</p>
              )}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

        <SidebarGroup>
          <SidebarGroupLabel>Tools</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              <SidebarMenuItem>
                <SidebarMenuButton disabled tooltip="Scan folder… (coming soon)">
                  <FolderSearchIcon />
                  <span>Scan folder…</span>
                </SidebarMenuButton>
                <SidebarMenuBadge>Soon</SidebarMenuBadge>
              </SidebarMenuItem>
              <SidebarMenuItem>
                <SidebarMenuButton disabled tooltip="Duplicates (coming soon)">
                  <CopyIcon />
                  <span>Duplicates</span>
                </SidebarMenuButton>
                <SidebarMenuBadge>Soon</SidebarMenuBadge>
              </SidebarMenuItem>
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

        <SidebarGroup className="group-data-[collapsible=icon]:hidden">
          <SidebarGroupLabel>Recent scans</SidebarGroupLabel>
          <SidebarGroupContent>
            <div className="flex items-center gap-2 px-2 py-1 text-sidebar-foreground/60 text-xs">
              <HistoryIcon className="size-3.5" />
              Scans from this session appear here.
            </div>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
      <SidebarRail />
    </Sidebar>
  );
}
