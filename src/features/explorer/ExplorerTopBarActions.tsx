import {
  CloudIcon,
  CloudOffIcon,
  PanelRightCloseIcon,
  PanelRightIcon,
  RotateCwIcon,
  SearchIcon,
  ZapIcon,
} from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Kbd, KbdGroup } from "@/components/ui/kbd";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { scanMethodLabel } from "@/features/scan/describeScanTarget";
import { useSearchPaletteStore } from "@/features/search/useSearchPaletteStore";
import type { ScanMethod } from "@/lib/backend/scans";
import { useExplorerSelectionStore } from "./useExplorerSelectionStore";
import { useScanAgeLabel } from "./useScanAgeLabel";

type ExplorerTopBarActionsProps = {
  method: ScanMethod;
  /** Short name of what was scanned, for example `C:`. */
  targetTitle: string;
  endedAtUnixMilliseconds: number | null;
  onRescan: () => void;
};

export function ExplorerTopBarActions({
  method,
  targetTitle,
  endedAtUnixMilliseconds,
  onRescan,
}: ExplorerTopBarActionsProps) {
  const openSearch = useSearchPaletteStore((state) => state.setOpen);
  const isInspectorOpen = useExplorerSelectionStore((state) => state.isInspectorOpen);
  const setInspectorOpen = useExplorerSelectionStore((state) => state.setInspectorOpen);
  const onlineOnlyVisibility = useExplorerSelectionStore((state) => state.onlineOnlyVisibility);
  const setOnlineOnlyVisibility = useExplorerSelectionStore(
    (state) => state.setOnlineOnlyVisibility,
  );
  const showsOnlineOnly = onlineOnlyVisibility === "shown";
  const onlineOnlyLabel = showsOnlineOnly ? "Hide online-only files" : "Show online-only files";
  const scanAgeLabel = useScanAgeLabel(endedAtUnixMilliseconds);

  return (
    <>
      {scanAgeLabel !== null && (
        <span className="hidden whitespace-nowrap text-muted-foreground text-xs xl:inline">
          {scanAgeLabel}
        </span>
      )}
      <Badge variant="secondary" className="hidden md:inline-flex">
        {method === "masterFileTable" && <ZapIcon />}
        {scanMethodLabel(method)}
      </Badge>
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant="outline"
            size="sm"
            className="w-44 justify-start text-muted-foreground max-2xl:w-auto"
            onClick={() => openSearch(true)}
          >
            <SearchIcon />
            <span className="max-2xl:sr-only">Search names…</span>
            <KbdGroup className="ml-auto max-2xl:hidden">
              <Kbd>Ctrl</Kbd>
              <Kbd>K</Kbd>
            </KbdGroup>
          </Button>
        </TooltipTrigger>
        <TooltipContent className="2xl:hidden">
          Search names
          <KbdGroup>
            <Kbd>Ctrl</Kbd>
            <Kbd>K</Kbd>
          </KbdGroup>
        </TooltipContent>
      </Tooltip>
      <Tooltip>
        <TooltipTrigger asChild>
          <Button variant="secondary" size="sm" onClick={onRescan}>
            <RotateCwIcon />
            Rescan
          </Button>
        </TooltipTrigger>
        <TooltipContent>
          Scan {targetTitle} again
          <KbdGroup>
            <Kbd>Ctrl</Kbd>
            <Kbd>R</Kbd>
          </KbdGroup>
        </TooltipContent>
      </Tooltip>
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant="ghost"
            size="icon"
            aria-label={onlineOnlyLabel}
            aria-pressed={showsOnlineOnly}
            onClick={() => setOnlineOnlyVisibility(showsOnlineOnly ? "hidden" : "shown")}
          >
            {showsOnlineOnly ? <CloudIcon /> : <CloudOffIcon />}
          </Button>
        </TooltipTrigger>
        <TooltipContent className="max-w-64">
          {onlineOnlyLabel}
          <span className="block text-muted-foreground">
            {showsOnlineOnly
              ? "Cloud files without data on this PC are listed; they still count as taking no space."
              : "Cloud files without data on this PC are left out, so sizes show what is really on disk."}
          </span>
        </TooltipContent>
      </Tooltip>
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant="ghost"
            size="icon"
            aria-label={isInspectorOpen ? "Hide details" : "Show details"}
            aria-pressed={isInspectorOpen}
            onClick={() => setInspectorOpen(!isInspectorOpen)}
          >
            {isInspectorOpen ? <PanelRightCloseIcon /> : <PanelRightIcon />}
          </Button>
        </TooltipTrigger>
        <TooltipContent>{isInspectorOpen ? "Hide details" : "Show details"}</TooltipContent>
      </Tooltip>
    </>
  );
}
