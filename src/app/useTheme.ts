import { useContext } from "react";
import { ThemeContext, type ThemeContextValue } from "./themeContext";

export function useTheme(): ThemeContextValue {
  const themeContextValue = useContext(ThemeContext);

  if (themeContextValue === undefined) {
    throw new Error("useTheme must be used inside ThemeProvider.");
  }

  return themeContextValue;
}
