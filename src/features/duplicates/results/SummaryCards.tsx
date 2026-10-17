import { formatBytesAsReadableSize } from "@/lib/formatting/formatBytesAsReadableSize";
import { formatCount } from "../formatting/formatCount";
import { pluralize } from "../formatting/pluralize";
import { SummaryCard } from "./SummaryCard";

type SummaryCardsProps = {
  groupCount: number;
  duplicateFileCount: number;
  wastedBytes: number;
  filesSelectedForRemoval: number;
  bytesSelectedForRemoval: number;
};

export function SummaryCards({
  groupCount,
  duplicateFileCount,
  wastedBytes,
  filesSelectedForRemoval,
  bytesSelectedForRemoval,
}: SummaryCardsProps) {
  return (
    <div className="grid gap-3 sm:grid-cols-3">
      <SummaryCard
        label="Duplicate groups"
        value={groupCount}
        formatValue={formatCount}
        caption={`${pluralize(duplicateFileCount, "file", "files")} with at least one copy`}
      />
      <SummaryCard
        label="Wasted space"
        value={wastedBytes}
        formatValue={formatBytesAsReadableSize}
        caption="Freed if only one copy of each file stays"
      />
      <SummaryCard
        label="Selected to free"
        value={bytesSelectedForRemoval}
        formatValue={formatBytesAsReadableSize}
        caption={`${pluralize(filesSelectedForRemoval, "copy", "copies")} marked to remove`}
        isHighlighted
      />
    </div>
  );
}
