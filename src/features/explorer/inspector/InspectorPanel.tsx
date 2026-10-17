import {
  ClipboardCopyIcon,
  ExternalLinkIcon,
  FolderInputIcon,
  FolderOpenIcon,
  Trash2Icon,
  TriangleAlertIcon,
} from "lucide-react";
import { motion } from "motion/react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { Skeleton } from "@/components/ui/skeleton";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import type { ScanResult } from "@/features/scan/useScanStore";
import type { ScanId } from "@/lib/backend/scans";
import type { NodeIndex } from "@/lib/backend/tree";
import { formatBytesAsReadableSize } from "@/lib/formatting/formatBytesAsReadableSize";
import { sidePanelVariants } from "@/lib/motion/transitions";
import { useLastSuccessfulData } from "@/lib/queryCache/useLastSuccessfulData";
import { nodeActionTarget } from "../actions/nodeActionTarget";
import { useNodeActions } from "../actions/useNodeActions";
import { FileTypeIcon } from "../FileTypeIcon";
import { useExplorerSelectionStore } from "../useExplorerSelectionStore";
import { useNodeDetails } from "../useTreeQueries";
import { describeItemDetails, describeScanDetails } from "./describeInspectorDetails";
import { canRecycleNode, describeNodeFlags } from "./describeNodeFlags";
import { InspectorDetailList } from "./InspectorDetailList";

type InspectorPanelProps = {
  scanId: ScanId;
  shownFolder: NodeIndex;
  result: ScanResult;
};

export function InspectorPanel({ scanId, shownFolder, result }: InspectorPanelProps) {
  const selectedNode = useExplorerSelectionStore((state) => state.selectedNode);
  const inspectedNode = selectedNode ?? shownFolder;
  const detailsState = useNodeDetails(scanId, inspectedNode).state;
  const details = useLastSuccessfulData(detailsState);
  const actions = useNodeActions(scanId);

  const isScanRoot = details?.node === result.rootNode;
  const target = details === null ? null : nodeActionTarget(details);
  const hasLocation = details !== null && details.fullPath !== null;

  return (
    <motion.aside
      aria-label="Details"
      className="flex w-72 shrink-0 flex-col gap-5 overflow-y-auto overflow-x-hidden border-l bg-card/40 p-5"
      variants={sidePanelVariants}
      initial="hidden"
      animate="visible"
      exit="exit"
    >
      {details === null && detailsState.status !== "error" && (
        <div className="grid gap-3" aria-busy>
          <Skeleton className="h-6 w-2/3" />
          <Skeleton className="h-10 w-1/2" />
          <Skeleton className="h-24 w-full" />
        </div>
      )}
      {detailsState.status === "error" && (
        <p className="text-muted-foreground text-sm">{detailsState.errorMessage}</p>
      )}
      {details !== null && target !== null && (
        <>
          <header className="grid gap-3">
            <div className="flex items-start gap-2.5">
              <FileTypeIcon
                kind={details.kind}
                category={details.category}
                isInaccessible={details.flags.inaccessible}
                className="mt-0.5 size-5"
              />
              <h2 className="min-w-0 font-semibold leading-snug [overflow-wrap:anywhere]">
                {details.name}
              </h2>
            </div>
            <p className="break-all font-mono text-muted-foreground text-xs leading-relaxed">
              {details.fullPath ?? "Not a single place on disk"}
            </p>
          </header>

          <div className="grid gap-0.5">
            <span className="font-semibold text-3xl tabular-nums tracking-tight">
              {formatBytesAsReadableSize(details.logicalSize)}
            </span>
            <span className="text-muted-foreground text-xs">
              {isScanRoot ? "in this scan" : "size"}
            </span>
          </div>

          {describeNodeFlags(details.flags).length > 0 && (
            <div className="flex flex-wrap gap-1.5">
              {describeNodeFlags(details.flags).map((badge) => (
                <Tooltip key={badge.key}>
                  <TooltipTrigger asChild>
                    <Badge variant={badge.tone === "warning" ? "destructive" : "outline"}>
                      {badge.tone === "warning" && <TriangleAlertIcon />}
                      {badge.label}
                    </Badge>
                  </TooltipTrigger>
                  <TooltipContent className="max-w-64">{badge.explanation}</TooltipContent>
                </Tooltip>
              ))}
            </div>
          )}

          <InspectorDetailList details={describeItemDetails(details)} />

          <div className="grid grid-cols-2 gap-2">
            {details.kind === "directory" && details.node !== shownFolder && (
              <Button
                variant="secondary"
                size="sm"
                className="col-span-2"
                onClick={() => actions.exploreFolder(target)}
              >
                <FolderInputIcon />
                Explore this folder
              </Button>
            )}
            <Button
              variant="outline"
              size="sm"
              disabled={!hasLocation}
              onClick={() => actions.openItem(target)}
            >
              {details.kind === "directory" ? <FolderOpenIcon /> : <ExternalLinkIcon />}
              Open
            </Button>
            <Button
              variant="outline"
              size="sm"
              disabled={!hasLocation}
              onClick={() => actions.revealItem(target)}
            >
              <FolderOpenIcon />
              Show
            </Button>
            <Button
              variant="outline"
              size="sm"
              disabled={!hasLocation}
              onClick={() => actions.copyPath(target)}
            >
              <ClipboardCopyIcon />
              Copy path
            </Button>
            <Button
              variant="outline"
              size="sm"
              className="text-destructive hover:text-destructive"
              disabled={!hasLocation || !canRecycleNode(details.flags, isScanRoot)}
              onClick={() => actions.requestRecycle(target)}
            >
              <Trash2Icon />
              Recycle…
            </Button>
          </div>

          {isScanRoot && (
            <>
              <Separator />
              <section className="grid gap-3">
                <h3 className="font-medium text-sm">About this scan</h3>
                <InspectorDetailList details={describeScanDetails(result)} />
                {result.currentTotals.inaccessibleDirectoryCount > 0 && (
                  <p className="text-muted-foreground text-xs leading-relaxed">
                    Some folders could not be read, so the real total may be larger. A scan as
                    administrator can usually read them.
                  </p>
                )}
              </section>
            </>
          )}
        </>
      )}
    </motion.aside>
  );
}
