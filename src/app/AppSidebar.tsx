import { CopyIcon, FolderSearchIcon, HouseIcon } from "lucide-react";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSkeleton,
  SidebarRail,
} from "@/components/ui/sidebar";
import { RecentScansSidebarGroup } from "@/features/scan/RecentScansSidebarGroup";
import { scanFolderFromPicker } from "@/features/scan/scanFolderFromPicker";
import { useOpenVolume } from "@/features/scan/useOpenVolume";
import { KeyboardShortcutsSidebarButton } from "@/features/shortcuts/KeyboardShortcutsSidebarButton";
import type { VolumesState } from "@/features/volumes/useVolumes";
import { VolumeSidebarItem } from "@/features/volumes/VolumeSidebarItem";
import { useNavigationStore } from "./useNavigationStore";

const SKELETON_ITEM_KEYS = ["first", "second"];

type AppSidebarProps = {
  volumesState: VolumesState;
};

export function AppSidebar({ volumesState }: AppSidebarProps) {
  const currentViewKind = useNavigationStore((state) => state.currentView.kind);
  const navigateToDuplicates = useNavigationStore((state) => state.navigateToDuplicates);
  const navigateHome = useNavigationStore((state) => state.navigateHome);
  const openVolume = useOpenVolume();

  return (
    <Sidebar collapsible="icon">
      <SidebarHeader>
        <button
          type="button"
          onClick={navigateHome}
          className="flex items-center gap-2 rounded-md px-1 py-1.5 text-left outline-none focus-visible:ring-2 focus-visible:ring-sidebar-ring"
        >
          <img src="/favicon.png" alt="" className="size-7 shrink-0 rounded-md" />
          <span className="truncate font-semibold group-data-[collapsible=icon]:hidden">
            Vexbyte Storage Map
          </span>
        </button>
      </SidebarHeader>

      <SidebarContent>
        <SidebarGroup className="pb-0">
          <SidebarGroupContent>
            <SidebarMenu>
              <SidebarMenuItem>
                <SidebarMenuButton
                  tooltip="Your drives (Ctrl+H)"
                  isActive={currentViewKind === "home"}
                  onClick={navigateHome}
                >
                  <HouseIcon />
                  <span>Your drives</span>
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

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
                  <VolumeSidebarItem
                    key={volume.mountPoint}
                    volume={volume}
                    onSelect={() => openVolume(volume.mountPoint)}
                  />
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
                <SidebarMenuButton
                  tooltip="Scan a folder"
                  onClick={() => void scanFolderFromPicker()}
                >
                  <FolderSearchIcon />
                  <span>Scan folder…</span>
                </SidebarMenuButton>
              </SidebarMenuItem>
              <SidebarMenuItem>
                <SidebarMenuButton
                  tooltip="Find duplicate files"
                  isActive={currentViewKind === "duplicates"}
                  onClick={navigateToDuplicates}
                >
                  <CopyIcon />
                  <span>Duplicates</span>
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

        <RecentScansSidebarGroup />
      </SidebarContent>
      <SidebarFooter>
        <KeyboardShortcutsSidebarButton />
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  );
}
