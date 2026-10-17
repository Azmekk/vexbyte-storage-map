import type { ReactNode } from "react";
import { Separator } from "@/components/ui/separator";
import { SidebarTrigger } from "@/components/ui/sidebar";
import { ThemeToggle } from "./ThemeToggle";

type TopBarProps = {
  /** Left side: a title or breadcrumbs. */
  children: ReactNode;
  /** Right side, before the theme toggle. */
  actions?: ReactNode;
};

export function TopBar({ children, actions }: TopBarProps) {
  return (
    <header className="sticky top-0 z-10 flex h-14 shrink-0 items-center gap-2 border-b bg-background/80 px-4 backdrop-blur">
      <SidebarTrigger className="-ml-1" />
      <Separator orientation="vertical" className="mr-2 data-[orientation=vertical]:h-4" />
      <div className="flex min-w-0 flex-1 items-center gap-2">{children}</div>
      <div className="flex shrink-0 items-center gap-1.5">
        {actions}
        <ThemeToggle />
      </div>
    </header>
  );
}
