import type { LucideIcon } from "lucide-react";
import type { ReactNode } from "react";
import { cn } from "@/lib/classNames";

type EmptyStateProps = {
  icon: LucideIcon;
  title: string;
  description: ReactNode;
  /** Buttons for the next step. */
  actions?: ReactNode;
  tone?: "neutral" | "warning";
  className?: string;
};

/** The shared layout for empty, cancelled and error states: what happened and what to do next. */
export function EmptyState({
  icon: Icon,
  title,
  description,
  actions,
  tone = "neutral",
  className,
}: EmptyStateProps) {
  return (
    <div
      role={tone === "warning" ? "alert" : undefined}
      className={cn("flex flex-col items-center gap-4 px-6 py-10 text-center", className)}
    >
      <div
        className={cn(
          "flex size-11 items-center justify-center rounded-full",
          tone === "warning"
            ? "bg-destructive/10 text-destructive"
            : "bg-muted text-muted-foreground",
        )}
      >
        <Icon className="size-5" aria-hidden />
      </div>
      <div className="grid max-w-md gap-1">
        <h2 className="font-semibold text-base">{title}</h2>
        <div className="text-muted-foreground text-sm">{description}</div>
      </div>
      {actions !== undefined && (
        <div className="flex flex-wrap justify-center gap-2">{actions}</div>
      )}
    </div>
  );
}
