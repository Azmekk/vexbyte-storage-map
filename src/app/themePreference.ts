export type ThemePreference = "system" | "light" | "dark";
export type ResolvedTheme = "light" | "dark";

export const THEME_PREFERENCES: readonly ThemePreference[] = ["system", "light", "dark"];

/** Also read by the inline script in `index.html` that applies the theme before first paint. */
export const THEME_PREFERENCE_STORAGE_KEY = "vsm.themePreference";

export function parseThemePreference(storedValue: string | null): ThemePreference {
  const knownPreference = THEME_PREFERENCES.find((preference) => preference === storedValue);
  return knownPreference ?? "system";
}

export function resolveTheme(
  themePreference: ThemePreference,
  systemPrefersDark: boolean,
): ResolvedTheme {
  if (themePreference === "system") {
    return systemPrefersDark ? "dark" : "light";
  }

  return themePreference;
}

export function readStoredThemePreference(): ThemePreference {
  try {
    return parseThemePreference(window.localStorage.getItem(THEME_PREFERENCE_STORAGE_KEY));
  } catch {
    return "system";
  }
}

export function storeThemePreference(themePreference: ThemePreference): void {
  try {
    window.localStorage.setItem(THEME_PREFERENCE_STORAGE_KEY, themePreference);
  } catch {
    // Storage can be unavailable; the preference then lasts for this session only.
  }
}
