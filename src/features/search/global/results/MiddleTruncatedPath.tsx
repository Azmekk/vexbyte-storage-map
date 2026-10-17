import { cn } from "@/lib/classNames";
import { splitPathForMiddleTruncation } from "./splitPathForMiddleTruncation";

type MiddleTruncatedPathProps = {
  path: string;
  className?: string;
};

/** A path that is shortened in the middle when it does not fit, with the full path on hover. */
export function MiddleTruncatedPath({ path, className }: MiddleTruncatedPathProps) {
  const { head, tail } = splitPathForMiddleTruncation(path);

  return (
    <span className={cn("flex min-w-0", className)} title={path}>
      <span className="truncate">{head}</span>
      {tail.length > 0 && <span className="shrink-0 whitespace-pre">{tail}</span>}
    </span>
  );
}
