import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { useScanLifecycle } from "@/features/scan/useScanLifecycle";
import { useVolumes } from "@/features/volumes/useVolumes";
import { AppSidebar } from "./AppSidebar";
import { CurrentViewScreen } from "./CurrentViewScreen";

export function AppLayout() {
  useScanLifecycle();
  const { volumesState, retryLoadingVolumes } = useVolumes();

  return (
    <SidebarProvider>
      <AppSidebar volumesState={volumesState} />
      <SidebarInset className="h-svh overflow-hidden">
        <CurrentViewScreen
          volumesState={volumesState}
          onRetryLoadingVolumes={retryLoadingVolumes}
        />
      </SidebarInset>
    </SidebarProvider>
  );
}
