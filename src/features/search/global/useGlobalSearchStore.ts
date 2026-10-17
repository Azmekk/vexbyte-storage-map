import { create } from "zustand";
import type { SearchSortOrder } from "@/lib/backend/search";

type GlobalSearchStoreState = {
  query: string;
  sortOrder: SearchSortOrder;
  /** The highlighted result row, or `null` while the focus is only in the search box. */
  selectedIndex: number | null;
  /** Bumped to ask the search box to take the focus (Ctrl+Shift+F while already on the screen). */
  inputFocusRequest: number;
  setQuery: (query: string) => void;
  clearQuery: () => void;
  setSortOrder: (sortOrder: SearchSortOrder) => void;
  selectIndex: (index: number | null) => void;
  requestInputFocus: () => void;
};

/** The Search screen's query, sort order and selection. The results live in `useGlobalSearch`. */
export const useGlobalSearchStore = create<GlobalSearchStoreState>((set) => ({
  query: "",
  sortOrder: "nameAscending",
  selectedIndex: null,
  inputFocusRequest: 0,
  setQuery: (query) => set({ query, selectedIndex: null }),
  clearQuery: () => set({ query: "", selectedIndex: null }),
  setSortOrder: (sortOrder) => set({ sortOrder, selectedIndex: null }),
  selectIndex: (selectedIndex) => set({ selectedIndex }),
  requestInputFocus: () => set((state) => ({ inputFocusRequest: state.inputFocusRequest + 1 })),
}));
