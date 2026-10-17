import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { SearchSortOrder } from "@/lib/backend/search";
import { isSearchSortOrder, SEARCH_SORT_OPTIONS } from "./searchSortOptions";

type SearchSortSelectProps = {
  sortOrder: SearchSortOrder;
  onChooseSortOrder: (sortOrder: SearchSortOrder) => void;
};

export function SearchSortSelect({ sortOrder, onChooseSortOrder }: SearchSortSelectProps) {
  return (
    <div className="flex items-center gap-2">
      <Label htmlFor="global-search-sort-order" className="text-muted-foreground text-xs">
        Sort by
      </Label>
      <Select
        value={sortOrder}
        onValueChange={(value) => {
          if (isSearchSortOrder(value)) {
            onChooseSortOrder(value);
          }
        }}
      >
        <SelectTrigger id="global-search-sort-order" size="sm" className="min-w-28">
          <SelectValue />
        </SelectTrigger>
        <SelectContent align="end">
          {SEARCH_SORT_OPTIONS.map((option) => (
            <SelectItem key={option.sortOrder} value={option.sortOrder}>
              {option.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}
