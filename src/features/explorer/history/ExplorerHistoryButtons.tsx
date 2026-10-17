import { ArrowLeftIcon, ArrowRightIcon } from "lucide-react";
import type { ScanId } from "@/lib/backend/scans";
import { HistoryNavigationButton } from "./HistoryNavigationButton";
import { useExplorerHistoryButtons } from "./useExplorerHistoryButtons";

type ExplorerHistoryButtonsProps = {
  scanId: ScanId;
};

/** Browser-style back and forward through the folders shown in this scan. */
export function ExplorerHistoryButtons({ scanId }: ExplorerHistoryButtonsProps) {
  const { back, forward } = useExplorerHistoryButtons(scanId);

  return (
    <nav aria-label="Folder history" className="flex items-center">
      <HistoryNavigationButton model={back} icon={ArrowLeftIcon} label="Back" shortcutKey="←" />
      <HistoryNavigationButton
        model={forward}
        icon={ArrowRightIcon}
        label="Forward"
        shortcutKey="→"
      />
    </nav>
  );
}
