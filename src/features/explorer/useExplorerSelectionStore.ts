import { create } from "zustand";
import type { ScanId } from "@/lib/backend/scans";
import type { ChildSortOrder, FileCategory, NodeIndex } from "@/lib/backend/tree";

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

  showScan: (scanId: ScanId, rootNode: NodeIndex) => void;
  openFolder: (folder: NodeIndex) => void;
  selectNode: (node: NodeIndex | null) => void;
  revealNode: (folder: NodeIndex, node: NodeIndex) => void;
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

  showScan: (scanId, rootNode) =>
    set((state) => {
      if (state.scanId === scanId && state.rootNode === rootNode) {
        return state;
      }
      return {
        scanId,
        rootNode,
        currentFolder: rootNode,
        selectedNode: null,
        expandedFolders: EMPTY_FOLDER_SET,
        lastExpandedFolder: null,
        childPageLimits: EMPTY_LIMIT_MAP,
        largestFilesCategory: "all",
      };
    }),

  openFolder: (folder) =>
    set((state) =>
      state.currentFolder === folder
        ? state
        : {
            currentFolder: folder,
            selectedNode: null,
            expandedFolders: EMPTY_FOLDER_SET,
            lastExpandedFolder: null,
          },
    ),

  selectNode: (node) => set({ selectedNode: node }),

  revealNode: (folder, node) =>
    set((state) => ({
      currentFolder: folder,
      selectedNode: node,
      expandedFolders: state.currentFolder === folder ? state.expandedFolders : EMPTY_FOLDER_SET,
    })),

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
