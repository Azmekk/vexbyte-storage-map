import type { ReactNode } from "react";
import { ThemeContext } from "./themeContext";
import { useThemeController } from "./useThemeController";

type ThemeProviderProps = {
  children: ReactNode;
};

/** Provides the system/light/dark theme preference, persisted in localStorage. */
export function ThemeProvider({ children }: ThemeProviderProps) {
  const themeContextValue = useThemeController();

  return <ThemeContext value={themeContextValue}>{children}</ThemeContext>;
}
