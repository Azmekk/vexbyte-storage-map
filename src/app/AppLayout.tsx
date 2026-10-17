import { ScrollArea } from "@/components/ui/scroll-area";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { HomeScreen } from "@/features/volumes/HomeScreen";
import { useVolumes } from "@/features/volumes/useVolumes";
import { AppSidebar } from "./AppSidebar";
import { TopBar } from "./TopBar";

export function AppLayout() {
  const { volumesState, retryLoadingVolumes } = useVolumes();

  return (
    <SidebarProvider>
      <AppSidebar volumesState={volumesState} />
      <SidebarInset className="h-svh overflow-hidden">
        <TopBar title="Home" />
        <ScrollArea className="min-h-0 flex-1">
          <HomeScreen volumesState={volumesState} onRetryLoadingVolumes={retryLoadingVolumes} />
        </ScrollArea>
      </SidebarInset>
    </SidebarProvider>
  );
}
