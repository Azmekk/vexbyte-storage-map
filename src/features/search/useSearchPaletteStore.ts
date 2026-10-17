import { create } from "zustand";

type SearchPaletteState = {
  isOpen: boolean;
  setOpen: (isOpen: boolean) => void;
};

export const useSearchPaletteStore = create<SearchPaletteState>((set) => ({
  isOpen: false,
  setOpen: (isOpen) => set({ isOpen }),
}));
