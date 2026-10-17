import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { DuplicateGroupSortOrder } from "@/lib/backend/generated";
import { isDuplicateGroupSortOrder, SORT_ORDER_OPTIONS } from "./sortOrderOptions";

type SortOrderSelectProps = {
  sortOrder: DuplicateGroupSortOrder;
  onChooseSortOrder: (sortOrder: DuplicateGroupSortOrder) => void;
};

export function SortOrderSelect({ sortOrder, onChooseSortOrder }: SortOrderSelectProps) {
  return (
    <div className="flex items-center gap-2">
      <Label htmlFor="duplicate-sort-order" className="text-muted-foreground">
        Sort by
      </Label>
      <Select
        value={sortOrder}
        onValueChange={(value) => {
          if (isDuplicateGroupSortOrder(value)) {
            onChooseSortOrder(value);
          }
        }}
      >
        <SelectTrigger id="duplicate-sort-order" className="min-w-44">
          <SelectValue />
        </SelectTrigger>
        <SelectContent align="end">
          {SORT_ORDER_OPTIONS.map((option) => (
            <SelectItem key={option.sortOrder} value={option.sortOrder}>
              {option.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}
