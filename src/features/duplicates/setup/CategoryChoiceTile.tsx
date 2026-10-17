import { CheckIcon } from "lucide-react";
import type { FileCategory } from "@/lib/backend/generated";
import { FileCategoryIcon } from "../shared/FileCategoryIcon";
import type { FileCategoryOption } from "./fileCategoryOptions";

type CategoryChoiceTileProps = {
  option: FileCategoryOption;
  isSelected: boolean;
  onToggle: (category: FileCategory) => void;
};

export function CategoryChoiceTile({ option, isSelected, onToggle }: CategoryChoiceTileProps) {
  return (
    <button
      type="button"
      aria-pressed={isSelected}
      onClick={() => onToggle(option.category)}
      data-selected={isSelected}
      className="group/tile relative flex items-center gap-3 rounded-xl border bg-card px-3 py-3 text-left outline-none transition-[background-color,border-color,transform] duration-150 ease-out hover:bg-muted/60 focus-visible:ring-3 focus-visible:ring-ring/50 active:translate-y-px data-[selected=true]:border-(--volume-used)/50 data-[selected=true]:bg-(--volume-used)/8"
    >
      <span className="flex size-9 shrink-0 items-center justify-center rounded-lg bg-muted text-muted-foreground transition-colors group-data-[selected=true]/tile:bg-(--volume-used) group-data-[selected=true]/tile:text-white">
        <FileCategoryIcon category={option.category} className="size-4.5" />
      </span>
      <span className="grid min-w-0 flex-1">
        <span className="truncate font-medium text-sm">{option.label}</span>
        <span className="truncate text-muted-foreground text-xs">{option.exampleExtensions}</span>
      </span>
      <span
        className="flex size-4 shrink-0 items-center justify-center rounded-[4px] border border-input transition-colors group-data-[selected=true]/tile:border-(--volume-used) group-data-[selected=true]/tile:bg-(--volume-used) group-data-[selected=true]/tile:text-white"
        aria-hidden
      >
        {isSelected && <CheckIcon className="size-3" />}
      </span>
    </button>
  );
}
