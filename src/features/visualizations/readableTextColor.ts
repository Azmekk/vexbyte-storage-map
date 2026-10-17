const DARK_INK = "#0b0b0b";
const LIGHT_INK = "#ffffff";

function parseHexColor(hexColor: string): [number, number, number] | null {
  const match = /^#?([0-9a-f]{6})$/i.exec(hexColor.trim());
  const hexDigits = match?.[1];

  if (hexDigits === undefined) {
    return null;
  }

  const value = Number.parseInt(hexDigits, 16);
  return [(value >> 16) & 0xff, (value >> 8) & 0xff, value & 0xff];
}

function linearChannel(channel: number): number {
  const normalized = channel / 255;
  return normalized <= 0.04045 ? normalized / 12.92 : ((normalized + 0.055) / 1.055) ** 2.4;
}

/** WCAG relative luminance, 0 for black to 1 for white. */
export function relativeLuminance(hexColor: string): number {
  const channels = parseHexColor(hexColor);

  if (channels === null) {
    return 0;
  }

  const [red, green, blue] = channels;
  return 0.2126 * linearChannel(red) + 0.7152 * linearChannel(green) + 0.0722 * linearChannel(blue);
}

function contrastRatio(firstLuminance: number, secondLuminance: number): number {
  const lighter = Math.max(firstLuminance, secondLuminance);
  const darker = Math.min(firstLuminance, secondLuminance);
  return (lighter + 0.05) / (darker + 0.05);
}

/** Black or white, whichever contrasts more with the fill a label sits on. */
export function readableTextColorOn(fillHexColor: string): string {
  const fillLuminance = relativeLuminance(fillHexColor);
  const contrastWithDark = contrastRatio(fillLuminance, relativeLuminance(DARK_INK));
  const contrastWithLight = contrastRatio(fillLuminance, 1);
  return contrastWithDark >= contrastWithLight ? DARK_INK : LIGHT_INK;
}
