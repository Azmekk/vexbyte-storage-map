/** Red, green and blue channels from 0 to 255. */
export type RgbChannels = [number, number, number];

/** Reads a `#rrggbb` colour; `null` for anything else. */
export function parseHexColor(hexColor: string): RgbChannels | null {
  const match = /^#?([0-9a-f]{6})$/i.exec(hexColor.trim());
  const hexDigits = match?.[1];

  if (hexDigits === undefined) {
    return null;
  }

  const value = Number.parseInt(hexDigits, 16);
  return [(value >> 16) & 0xff, (value >> 8) & 0xff, value & 0xff];
}
