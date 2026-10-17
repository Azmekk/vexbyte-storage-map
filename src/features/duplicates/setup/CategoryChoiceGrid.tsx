import type { FileCategory } from "@/lib/backend/generated";
import { CategoryChoiceTile } from "./CategoryChoiceTile";
import { FILE_CATEGORY_OPTIONS } from "./fileCategoryOptions";

type CategoryChoiceGridProps = {
  selectedCategories: readonly FileCategory[];
  onToggleCategory: (category: FileCategory) => void;
};

export function CategoryChoiceGrid({
  selectedCategories,
  onToggleCategory,
}: CategoryChoiceGridProps) {
  return (
    <section className="grid content-start gap-3" aria-labelledby="duplicate-categories-heading">
      <div className="grid gap-0.5">
        <h2 id="duplicate-categories-heading" className="font-semibold text-base">
          What to compare
        </h2>
        <p className="text-muted-foreground text-sm">
          Only exact copies count: same content, byte for byte.
        </p>
      </div>
      <div className="grid gap-2 sm:grid-cols-2">
        {FILE_CATEGORY_OPTIONS.map((option) => (
          <CategoryChoiceTile
            key={option.category}
            option={option}
            isSelected={selectedCategories.includes(option.category)}
            onToggle={onToggleCategory}
          />
        ))}
      </div>
    </section>
  );
}
