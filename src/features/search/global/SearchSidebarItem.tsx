import { SearchIcon } from "lucide-react";
import { useNavigationStore } from "@/app/useNavigationStore";
import { SidebarMenuButton, SidebarMenuItem } from "@/components/ui/sidebar";
import { describeIndexStatusIndicator } from "./status/describeIndexStatusIndicator";
import { SearchIndexStatusIndicator } from "./status/SearchIndexStatusIndicator";
import { useSearchIndexStatus } from "./status/useSearchIndexStatus";

export function SearchSidebarItem() {
  const isActive = useNavigationStore((state) => state.currentView.kind === "search");
  const navigateToSearch = useNavigationStore((state) => state.navigateToSearch);
  const { status, loadErrorMessage } = useSearchIndexStatus();
  const indicator = describeIndexStatusIndicator(status, loadErrorMessage);
  const tooltip =
    indicator.kind === "warming"
      ? "Search files (Ctrl+Shift+F), warming up"
      : indicator.kind === "warning"
        ? `Search files (Ctrl+Shift+F). ${indicator.tooltip}`
        : "Search files (Ctrl+Shift+F)";

  return (
    <SidebarMenuItem>
      <SidebarMenuButton tooltip={tooltip} isActive={isActive} onClick={navigateToSearch}>
        <SearchIcon />
        <span className="flex-1">Search files</span>
        <SearchIndexStatusIndicator indicator={indicator} />
      </SidebarMenuButton>
    </SidebarMenuItem>
  );
}
