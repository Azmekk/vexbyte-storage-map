import type { HierarchyNode } from "d3-hierarchy";
import type { FileCategory, VisualizationNode } from "@/lib/backend/tree";
import { FILE_CATEGORY_ORDER } from "./categoryPalette";

/** One slot per known category plus a last slot for files of no known category. */
const OTHER_FILES_SLOT = FILE_CATEGORY_ORDER.length;

/**
 * For every folder of the layout, the known category whose files take the most bytes inside it,
 * counting only the files present in the pruned subtree. `null` when files of no known category
 * win or the folder has no listed files. One pass from the leaves up, so it stays cheap at the
 * node cap.
 */
export function dominantFileCategories(
  root: HierarchyNode<VisualizationNode>,
): Map<HierarchyNode<VisualizationNode>, FileCategory | null> {
  const bytesPerCategory = new Map<HierarchyNode<VisualizationNode>, Float64Array>();
  const dominantCategories = new Map<HierarchyNode<VisualizationNode>, FileCategory | null>();

  root.eachAfter((node) => {
    const children = node.children ?? [];
    const totals = new Float64Array(OTHER_FILES_SLOT + 1);

    if (children.length === 0) {
      if (node.data.kind === "file") {
        const category = node.data.category;
        const slot = category === null ? OTHER_FILES_SLOT : FILE_CATEGORY_ORDER.indexOf(category);
        totals[slot === -1 ? OTHER_FILES_SLOT : slot] = Math.max(0, node.data.logicalSize);
      }
      bytesPerCategory.set(node, totals);
      return;
    }

    for (const child of children) {
      const childTotals = bytesPerCategory.get(child);
      if (childTotals !== undefined) {
        for (let slot = 0; slot < totals.length; slot += 1) {
          totals[slot] = (totals[slot] ?? 0) + (childTotals[slot] ?? 0);
        }
      }
      bytesPerCategory.delete(child);
    }
    bytesPerCategory.set(node, totals);

    let winningSlot = OTHER_FILES_SLOT;
    let winningBytes = 0;
    for (let slot = 0; slot < totals.length; slot += 1) {
      const slotBytes = totals[slot] ?? 0;
      if (slotBytes > winningBytes) {
        winningSlot = slot;
        winningBytes = slotBytes;
      }
    }
    dominantCategories.set(node, FILE_CATEGORY_ORDER[winningSlot] ?? null);
  });

  return dominantCategories;
}
