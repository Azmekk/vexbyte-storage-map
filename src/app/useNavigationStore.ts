import { create } from "zustand";
import type { ScanId } from "@/lib/backend/scans";

/** The screen shown in the main area. */
export type CurrentView =
  | { kind: "home" }
  | { kind: "scan"; scanId: ScanId }
  | { kind: "duplicates" }
  | { kind: "search" };

type NavigationState = {
  currentView: CurrentView;
  navigateHome: () => void;
  navigateToScan: (scanId: ScanId) => void;
  navigateToDuplicates: () => void;
  navigateToSearch: () => void;
};

/** Which screen the app shows. Shared by every feature; keep its shape stable. */
export const useNavigationStore = create<NavigationState>((set) => ({
  currentView: { kind: "home" },
  navigateHome: () => set({ currentView: { kind: "home" } }),
  navigateToScan: (scanId) => set({ currentView: { kind: "scan", scanId } }),
  navigateToDuplicates: () => set({ currentView: { kind: "duplicates" } }),
  navigateToSearch: () => set({ currentView: { kind: "search" } }),
}));
