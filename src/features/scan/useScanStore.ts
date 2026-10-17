import { create } from "zustand";
import type {
  ScanId,
  ScanMethod,
  ScanOutcome,
  ScanOverview,
  ScanProgressUpdate,
  ScanStatus,
  ScanSummary,
  ScanTarget,
  TreeTotals,
} from "@/lib/backend/scans";
import type { NodeIndex } from "@/lib/backend/tree";

/** What the explorer needs once a scan finished. */
export type ScanResult = {
  rootNode: NodeIndex;
  summary: ScanSummary;
  currentTotals: TreeTotals;
};

/** One scan of this session, as the UI tracks it. */
export type ScanRecord = {
  scanId: ScanId;
  target: ScanTarget;
  method: ScanMethod;
  status: ScanStatus;
  latestProgress: ScanProgressUpdate | null;
  outcome: ScanOutcome | null;
  result: ScanResult | null;
};

type ScanStoreState = {
  scans: Record<ScanId, ScanRecord>;
  /** Outcomes that arrived before `startScan` returned the scan's id. */
  outcomesOfUnregisteredScans: Record<ScanId, ScanOutcome>;
  registerStartedScan: (
    scanId: ScanId,
    target: ScanTarget,
    method: ScanMethod,
    initialProgress: ScanProgressUpdate | null,
  ) => void;
  recordProgress: (scanId: ScanId, progress: ScanProgressUpdate) => void;
  recordOutcome: (scanId: ScanId, outcome: ScanOutcome) => void;
  recordResult: (scanId: ScanId, result: ScanResult) => void;
  updateCurrentTotals: (scanId: ScanId, totals: TreeTotals) => void;
  mergeOverviews: (overviews: ScanOverview[]) => void;
  removeScan: (scanId: ScanId) => void;
};

function statusOfOutcome(outcome: ScanOutcome): ScanStatus {
  return outcome.kind;
}

function recordFromOverview(overview: ScanOverview): ScanRecord {
  const state = overview.state;
  const result: ScanResult | null =
    state.kind === "finished"
      ? { rootNode: state.rootNode, summary: state.summary, currentTotals: state.currentTotals }
      : null;
  let outcome: ScanOutcome | null = null;

  if (state.kind === "finished") {
    outcome = { kind: "finished", summary: state.summary };
  } else if (state.kind === "cancelled") {
    outcome = { kind: "cancelled" };
  } else if (state.kind === "failed") {
    outcome = { kind: "failed", failure: state.failure };
  }

  return {
    scanId: overview.scanId,
    target: overview.target,
    method: overview.method,
    status: overview.status,
    latestProgress: null,
    outcome,
    result,
  };
}

/** Every scan of this session: its status, latest progress and, once finished, its root. */
export const useScanStore = create<ScanStoreState>((set) => ({
  scans: {},
  outcomesOfUnregisteredScans: {},

  registerStartedScan: (scanId, target, method, initialProgress) =>
    set((state) => {
      const earlyOutcome = state.outcomesOfUnregisteredScans[scanId] ?? null;
      const { [scanId]: _consumedOutcome, ...remainingOutcomes } =
        state.outcomesOfUnregisteredScans;
      const record: ScanRecord = {
        scanId,
        target,
        method,
        status: earlyOutcome === null ? "running" : statusOfOutcome(earlyOutcome),
        latestProgress: initialProgress,
        outcome: earlyOutcome,
        result: state.scans[scanId]?.result ?? null,
      };

      return {
        scans: { ...state.scans, [scanId]: record },
        outcomesOfUnregisteredScans: remainingOutcomes,
      };
    }),

  recordProgress: (scanId, progress) =>
    set((state) => {
      const record = state.scans[scanId];
      if (record === undefined || record.status !== "running") {
        return state;
      }
      return { scans: { ...state.scans, [scanId]: { ...record, latestProgress: progress } } };
    }),

  recordOutcome: (scanId, outcome) =>
    set((state) => {
      const record = state.scans[scanId];
      if (record === undefined) {
        return {
          outcomesOfUnregisteredScans: { ...state.outcomesOfUnregisteredScans, [scanId]: outcome },
        };
      }
      const updatedRecord: ScanRecord = { ...record, outcome, status: statusOfOutcome(outcome) };
      return { scans: { ...state.scans, [scanId]: updatedRecord } };
    }),

  recordResult: (scanId, result) =>
    set((state) => {
      const record = state.scans[scanId];
      if (record === undefined) {
        return state;
      }
      return { scans: { ...state.scans, [scanId]: { ...record, result, status: "finished" } } };
    }),

  updateCurrentTotals: (scanId, totals) =>
    set((state) => {
      const record = state.scans[scanId];
      if (record?.result == null) {
        return state;
      }
      const result = { ...record.result, currentTotals: totals };
      return { scans: { ...state.scans, [scanId]: { ...record, result } } };
    }),

  mergeOverviews: (overviews) =>
    set((state) => {
      const scans = { ...state.scans };
      for (const overview of overviews) {
        const existingRecord = scans[overview.scanId];
        const isAlreadyUpToDate =
          existingRecord !== undefined &&
          existingRecord.status === overview.status &&
          (overview.status !== "finished" || existingRecord.result !== null);

        if (!isAlreadyUpToDate) {
          scans[overview.scanId] = {
            ...recordFromOverview(overview),
            latestProgress: existingRecord?.latestProgress ?? null,
          };
        }
      }
      return { scans };
    }),

  removeScan: (scanId) =>
    set((state) => {
      const { [scanId]: _removedRecord, ...remainingScans } = state.scans;
      return { scans: remainingScans };
    }),
}));
