import { parseHexColor } from "./parseHexColor";

function channelToHex(channel: number): string {
  return Math.round(Math.min(Math.max(channel, 0), 255))
    .toString(16)
    .padStart(2, "0");
}

/** Blends `toColor` into `fromColor`; `amount` 0 keeps `fromColor`, 1 gives `toColor`. */
export function mixHexColors(fromColor: string, toColor: string, amount: number): string {
  const fromChannels = parseHexColor(fromColor);
  const toChannels = parseHexColor(toColor);

  if (fromChannels === null || toChannels === null) {
    return fromColor;
  }

  const mixedChannels = fromChannels.map(
    (fromChannel, channelIndex) =>
      fromChannel + ((toChannels[channelIndex] ?? fromChannel) - fromChannel) * amount,
  );
  return `#${mixedChannels.map(channelToHex).join("")}`;
}
