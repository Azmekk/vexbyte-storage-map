import { create } from "zustand";
import type { ScanId } from "@/lib/backend/scans";
import type { NodeIndex } from "@/lib/backend/tree";

type RecycleConfirmationState = {
  scanId: ScanId | null;
  /** Items waiting for the user to confirm; empty when the dialog is closed. */
  pendingNodes: NodeIndex[];
  requestRecycle: (scanId: ScanId, nodes: NodeIndex[]) => void;
  closeConfirmation: () => void;
};

/** The "Move to Recycle Bin?" dialog: which items it asks about. */
export const useRecycleConfirmationStore = create<RecycleConfirmationState>((set) => ({
  scanId: null,
  pendingNodes: [],
  requestRecycle: (scanId, nodes) => set({ scanId, pendingNodes: nodes }),
  closeConfirmation: () => set({ pendingNodes: [] }),
}));
