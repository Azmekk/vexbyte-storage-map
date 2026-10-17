import { create } from "zustand";
import type { ScanId } from "@/lib/backend/scans";
import type { ChildSortOrder, FileCategory, NodeIndex } from "@/lib/backend/tree";
import {
  createNavigationHistory,
  currentHistoryEntry,
  goBackInHistory,
  goForwardInHistory,
  type NavigationHistory,
  pushHistoryEntry,
  replaceCurrentHistoryEntry,
} from "./history/navigationHistory";

export type VisualizationTab = "treemap" | "sunburst" | "fileTypes" | "largestFiles";
export type TreemapColorMode = "category" | "depth";

/** `"all"` shows every category in the largest files list. */
export type LargestFilesCategoryFilter = FileCategory | "all";

const VISUALIZATION_TABS: readonly string[] = ["treemap", "sunburst", "fileTypes", "largestFiles"];
const TREEMAP_COLOR_MODES: readonly string[] = ["category", "depth"];

/** Narrows a value reported by the tabs component. */
export function isVisualizationTab(value: string): value is VisualizationTab {
  return VISUALIZATION_TABS.includes(value);
}

/** Narrows a value reported by the colour mode toggle. */
export function isTreemapColorMode(value: string): value is TreemapColorMode {
  return TREEMAP_COLOR_MODES.includes(value);
}

export const DEFAULT_CHILD_PAGE_LIMIT = 300;
const CHILD_PAGE_LIMIT_STEP = 1000;

export type NavigateToNodeOptions = {
  /** The item to select in the opened folder; by default nothing (or the current selection). */
  selectedNode?: NodeIndex | null;
  /** Corrects the current history entry instead of adding a step (for a folder that is gone). */
  replaceHistory?: boolean;
};

type ExplorerSelectionState = {
  scanId: ScanId | null;
  rootNode: NodeIndex | null;
  /** The folder the tree table, breadcrumbs and visualizations show. */
  currentFolder: NodeIndex | null;
  /** The item the inspector shows; `null` means the current folder. */
  selectedNode: NodeIndex | null;
  expandedFolders: ReadonlySet<NodeIndex>;
  /** Folders expanded most recently, so only their new rows animate in. */
  lastExpandedFolder: NodeIndex | null;
  childPageLimits: ReadonlyMap<NodeIndex, number>;
  sortOrder: ChildSortOrder;
  isInspectorOpen: boolean;
  visualizationTab: VisualizationTab;
  treemapColorMode: TreemapColorMode;
  largestFilesCategory: LargestFilesCategoryFilter;
  /** Back and forward history of the shown folder, one per scan of this session. */
  navigationHistories: ReadonlyMap<ScanId, NavigationHistory>;

  showScan: (scanId: ScanId, rootNode: NodeIndex) => void;
  /** Shows `folder` and records it in the history. Every way of changing folder goes through it. */
  navigateToNode: (folder: NodeIndex, options?: NavigateToNodeOptions) => void;
  goBack: () => void;
  goForward: () => void;
  selectNode: (node: NodeIndex | null) => void;
  toggleFolderExpanded: (folder: NodeIndex) => void;
  setFolderExpanded: (folder: NodeIndex, isExpanded: boolean) => void;
  showMoreChildren: (folder: NodeIndex) => void;
  clearLastExpandedFolder: () => void;
  setSortOrder: (sortOrder: ChildSortOrder) => void;
  setInspectorOpen: (isOpen: boolean) => void;
  setVisualizationTab: (tab: VisualizationTab) => void;
  setTreemapColorMode: (colorMode: TreemapColorMode) => void;
  showLargestFilesOfCategory: (category: LargestFilesCategoryFilter) => void;
  setLargestFilesCategory: (category: LargestFilesCategoryFilter) => void;
};

const EMPTY_FOLDER_SET: ReadonlySet<NodeIndex> = new Set();
const EMPTY_LIMIT_MAP: ReadonlyMap<NodeIndex, number> = new Map();

/** The history of the shown scan, or `null` before a scan is shown. */
export function selectCurrentHistory(state: ExplorerSelectionState): NavigationHistory | null {
  return state.scanId === null ? null : (state.navigationHistories.get(state.scanId) ?? null);
}

function withHistory(
  state: ExplorerSelectionState,
  history: NavigationHistory,
): ReadonlyMap<ScanId, NavigationHistory> {
  const navigationHistories = new Map(state.navigationHistories);
  if (state.scanId !== null) {
    navigationHistories.set(state.scanId, history);
  }
  return navigationHistories;
}

function showFolderFromHistory(
  state: ExplorerSelectionState,
  history: NavigationHistory,
): Partial<ExplorerSelectionState> {
  const folder = currentHistoryEntry(history) ?? state.rootNode;
  return {
    navigationHistories: withHistory(state, history),
    currentFolder: folder,
    selectedNode: null,
    expandedFolders: EMPTY_FOLDER_SET,
    lastExpandedFolder: null,
  };
}

/** Where the user is in the explorer of the shown scan. Holds ids and UI state only. */
export const useExplorerSelectionStore = create<ExplorerSelectionState>((set) => ({
  scanId: null,
  rootNode: null,
  currentFolder: null,
  selectedNode: null,
  expandedFolders: EMPTY_FOLDER_SET,
  lastExpandedFolder: null,
  childPageLimits: EMPTY_LIMIT_MAP,
  sortOrder: "logicalSizeDescending",
  isInspectorOpen: true,
  visualizationTab: "treemap",
  treemapColorMode: "category",
  largestFilesCategory: "all",
  navigationHistories: new Map(),

  showScan: (scanId, rootNode) =>
    set((state) => {
      if (state.scanId === scanId && state.rootNode === rootNode) {
        return state;
      }
      const navigationHistories = new Map(state.navigationHistories);
      const savedHistory = navigationHistories.get(scanId);
      const history = savedHistory ?? createNavigationHistory(rootNode);
      navigationHistories.set(scanId, history);
      return {
        scanId,
        rootNode,
        currentFolder: currentHistoryEntry(history) ?? rootNode,
        selectedNode: null,
        expandedFolders: EMPTY_FOLDER_SET,
        lastExpandedFolder: null,
        childPageLimits: EMPTY_LIMIT_MAP,
        largestFilesCategory: "all",
        navigationHistories,
      };
    }),

  navigateToNode: (folder, options = {}) =>
    set((state) => {
      const isSameFolder = state.currentFolder === folder;
      const keptSelection = isSameFolder ? state.selectedNode : null;
      const selectedNode =
        options.selectedNode === undefined ? keptSelection : options.selectedNode;
      const history = selectCurrentHistory(state) ?? createNavigationHistory(folder);
      const updatedHistory = options.replaceHistory
        ? replaceCurrentHistoryEntry(history, folder)
        : pushHistoryEntry(history, folder);
      const navigationHistories =
        updatedHistory === history ? state.navigationHistories : withHistory(state, updatedHistory);

      if (isSameFolder) {
        return state.selectedNode === selectedNode && updatedHistory === history
          ? state
          : { selectedNode, navigationHistories };
      }
      return {
        currentFolder: folder,
        selectedNode,
        expandedFolders: EMPTY_FOLDER_SET,
        lastExpandedFolder: null,
        navigationHistories,
      };
    }),

  goBack: () =>
    set((state) => {
      const history = selectCurrentHistory(state);
      const updatedHistory = history === null ? null : goBackInHistory(history);
      return updatedHistory === null || updatedHistory === history
        ? state
        : showFolderFromHistory(state, updatedHistory);
    }),

  goForward: () =>
    set((state) => {
      const history = selectCurrentHistory(state);
      const updatedHistory = history === null ? null : goForwardInHistory(history);
      return updatedHistory === null || updatedHistory === history
        ? state
        : showFolderFromHistory(state, updatedHistory);
    }),

  selectNode: (node) => set({ selectedNode: node }),

  toggleFolderExpanded: (folder) =>
    set((state) => {
      const expandedFolders = new Set(state.expandedFolders);
      const willExpand = !expandedFolders.has(folder);
      if (willExpand) {
        expandedFolders.add(folder);
      } else {
        expandedFolders.delete(folder);
      }
      return { expandedFolders, lastExpandedFolder: willExpand ? folder : null };
    }),

  setFolderExpanded: (folder, isExpanded) =>
    set((state) => {
      if (state.expandedFolders.has(folder) === isExpanded) {
        return state;
      }
      const expandedFolders = new Set(state.expandedFolders);
      if (isExpanded) {
        expandedFolders.add(folder);
      } else {
        expandedFolders.delete(folder);
      }
      return { expandedFolders, lastExpandedFolder: isExpanded ? folder : null };
    }),

  showMoreChildren: (folder) =>
    set((state) => {
      const childPageLimits = new Map(state.childPageLimits);
      const currentLimit = childPageLimits.get(folder) ?? DEFAULT_CHILD_PAGE_LIMIT;
      childPageLimits.set(folder, currentLimit + CHILD_PAGE_LIMIT_STEP);
      return { childPageLimits };
    }),

  clearLastExpandedFolder: () => set({ lastExpandedFolder: null }),
  setSortOrder: (sortOrder) => set({ sortOrder, lastExpandedFolder: null }),
  setInspectorOpen: (isInspectorOpen) => set({ isInspectorOpen }),
  setVisualizationTab: (visualizationTab) => set({ visualizationTab }),
  setTreemapColorMode: (treemapColorMode) => set({ treemapColorMode }),
  showLargestFilesOfCategory: (largestFilesCategory) =>
    set({ largestFilesCategory, visualizationTab: "largestFiles" }),
  setLargestFilesCategory: (largestFilesCategory) => set({ largestFilesCategory }),
}));
