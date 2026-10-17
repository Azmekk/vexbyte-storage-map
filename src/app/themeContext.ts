import { createContext } from "react";
import type { ResolvedTheme, ThemePreference } from "./themePreference";

export type ThemeContextValue = {
  themePreference: ThemePreference;
  resolvedTheme: ResolvedTheme;
  setThemePreference: (themePreference: ThemePreference) => void;
};

export const ThemeContext = createContext<ThemeContextValue | undefined>(undefined);
