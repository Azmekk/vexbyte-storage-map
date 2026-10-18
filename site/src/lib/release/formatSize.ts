export function formatMegabytes(bytes: number): string {
  return `${(bytes / 1_048_576).toFixed(1)} MB`;
}
