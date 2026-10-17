import {
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

type ExplorerTopBarActionsProps = {
  method: ScanMethod;
  onRescan: () => void;
};

export function ExplorerTopBarActions({ method, onRescan }: ExplorerTopBarActionsProps) {
  const openSearch = useSearchPaletteStore((state) => state.setOpen);
  const isInspectorOpen = useExplorerSelectionStore((state) => state.isInspectorOpen);
  const setInspectorOpen = useExplorerSelectionStore((state) => state.setInspectorOpen);

  return (
    <>
      <Badge variant="secondary" className="hidden md:inline-flex">
        {method === "masterFileTable" && <ZapIcon />}
        {scanMethodLabel(method)}
      </Badge>
      <Button
        variant="outline"
        size="sm"
        className="w-44 justify-start text-muted-foreground max-lg:w-auto"
        onClick={() => openSearch(true)}
      >
        <SearchIcon />
        <span className="max-lg:sr-only">Search names…</span>
        <KbdGroup className="ml-auto max-lg:hidden">
          <Kbd>Ctrl</Kbd>
          <Kbd>K</Kbd>
        </KbdGroup>
      </Button>
      <Tooltip>
        <TooltipTrigger asChild>
          <Button variant="ghost" size="icon" aria-label="Scan again" onClick={onRescan}>
            <RotateCwIcon />
          </Button>
        </TooltipTrigger>
        <TooltipContent>Scan again</TooltipContent>
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
