const KILOBYTE = 1024;
const MEGABYTE = 1024 * KILOBYTE;

export type MinimumSizePreset = {
  bytes: number;
  label: string;
};

export const MINIMUM_SIZE_PRESETS: readonly MinimumSizePreset[] = [
  { bytes: 64 * KILOBYTE, label: "64 KB" },
  { bytes: 256 * KILOBYTE, label: "256 KB" },
  { bytes: MEGABYTE, label: "1 MB" },
  { bytes: 10 * MEGABYTE, label: "10 MB" },
  { bytes: 100 * MEGABYTE, label: "100 MB" },
];

export const DEFAULT_MINIMUM_FILE_SIZE_BYTES = 64 * KILOBYTE;

/** The Select needs string values; a preset is identified by its byte count. */
export function minimumSizePresetValue(preset: MinimumSizePreset): string {
  return String(preset.bytes);
}

export function defaultMinimumSizePreset(): MinimumSizePreset {
  const defaultPreset = MINIMUM_SIZE_PRESETS.find(
    (preset) => preset.bytes === DEFAULT_MINIMUM_FILE_SIZE_BYTES,
  );

  return defaultPreset ?? { bytes: DEFAULT_MINIMUM_FILE_SIZE_BYTES, label: "64 KB" };
}

/** The preset a Select value names, or the default preset for anything unknown. */
export function minimumSizePresetFromValue(value: string): MinimumSizePreset {
  const bytes = Number(value);
  const matchingPreset = MINIMUM_SIZE_PRESETS.find((preset) => preset.bytes === bytes);

  return matchingPreset ?? defaultMinimumSizePreset();
}
