import { create } from "zustand";

type KeyboardShortcutsDialogState = {
  isOpen: boolean;
  setOpen: (isOpen: boolean) => void;
};

export const useKeyboardShortcutsDialogStore = create<KeyboardShortcutsDialogState>((set) => ({
  isOpen: false,
  setOpen: (isOpen) => set({ isOpen }),
}));
