import { useCallback, useLayoutEffect, useMemo, useState } from "react";
import type { ThemeContextValue } from "./themeContext";
import {
  readStoredThemePreference,
  resolveTheme,
  storeThemePreference,
  type ThemePreference,
} from "./themePreference";
import { useSystemPrefersDark } from "./useSystemPrefersDark";

/** Owns the theme preference, persists it and applies the resolved theme to the document. */
export function useThemeController(): ThemeContextValue {
  const [themePreference, setThemePreferenceState] =
    useState<ThemePreference>(readStoredThemePreference);
  const systemPrefersDark = useSystemPrefersDark();
  const resolvedTheme = resolveTheme(themePreference, systemPrefersDark);

  useLayoutEffect(() => {
    const rootElement = document.documentElement;
    rootElement.classList.toggle("dark", resolvedTheme === "dark");
    rootElement.style.colorScheme = resolvedTheme;
  }, [resolvedTheme]);

  const setThemePreference = useCallback((nextThemePreference: ThemePreference) => {
    setThemePreferenceState(nextThemePreference);
    storeThemePreference(nextThemePreference);
  }, []);

  return useMemo(
    () => ({ themePreference, resolvedTheme, setThemePreference }),
    [themePreference, resolvedTheme, setThemePreference],
  );
}
