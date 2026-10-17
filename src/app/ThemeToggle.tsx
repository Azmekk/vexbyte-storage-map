import { CheckIcon, MonitorIcon, MoonIcon, SunIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import type { ThemePreference } from "./themePreference";
import { useTheme } from "./useTheme";

const THEME_OPTIONS: { preference: ThemePreference; label: string; icon: typeof SunIcon }[] = [
  { preference: "system", label: "System", icon: MonitorIcon },
  { preference: "light", label: "Light", icon: SunIcon },
  { preference: "dark", label: "Dark", icon: MoonIcon },
];

export function ThemeToggle() {
  const { themePreference, resolvedTheme, setThemePreference } = useTheme();
  const CurrentThemeIcon = resolvedTheme === "dark" ? MoonIcon : SunIcon;

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="icon" aria-label="Change theme">
          <CurrentThemeIcon />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        {THEME_OPTIONS.map(({ preference, label, icon: OptionIcon }) => (
          <DropdownMenuItem key={preference} onSelect={() => setThemePreference(preference)}>
            <OptionIcon />
            {label}
            {preference === themePreference && <CheckIcon className="ml-auto" />}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
