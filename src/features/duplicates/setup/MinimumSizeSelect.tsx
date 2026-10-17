import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  MINIMUM_SIZE_PRESETS,
  minimumSizePresetFromValue,
  minimumSizePresetValue,
} from "./minimumSizePresets";

type MinimumSizeSelectProps = {
  minimumFileSizeBytes: number;
  onMinimumFileSizeChange: (bytes: number) => void;
};

export function MinimumSizeSelect({
  minimumFileSizeBytes,
  onMinimumFileSizeChange,
}: MinimumSizeSelectProps) {
  return (
    <div className="flex flex-wrap items-center justify-between gap-3 rounded-xl border bg-card px-4 py-3">
      <div className="grid gap-0.5">
        <Label htmlFor="duplicate-minimum-size">Skip files smaller than</Label>
        <p className="text-muted-foreground text-xs">
          Small files free little space and slow the search down.
        </p>
      </div>
      <Select
        value={String(minimumFileSizeBytes)}
        onValueChange={(value) => onMinimumFileSizeChange(minimumSizePresetFromValue(value).bytes)}
      >
        <SelectTrigger id="duplicate-minimum-size" className="w-28">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {MINIMUM_SIZE_PRESETS.map((preset) => (
            <SelectItem key={preset.bytes} value={minimumSizePresetValue(preset)}>
              {preset.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}
