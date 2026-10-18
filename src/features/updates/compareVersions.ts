/** `1.2.3` as `[1, 2, 3]`, ignoring a `v` prefix and a `-beta.1` or `+build` suffix. */
function parseVersionComponents(version: string): number[] | null {
  const coreVersion = version.trim().replace(/^v/i, "").split(/[-+]/)[0] ?? "";
  const components = coreVersion.split(".");

  if (coreVersion.length === 0 || components.some((component) => !/^\d+$/.test(component))) {
    return null;
  }

  return components.map(Number);
}

/**
 * Negative when `left` is older, positive when newer, zero when equal; `null` when either is not
 * a version number. Components compare as numbers, and missing ones count as zero.
 */
export function compareVersions(left: string, right: string): number | null {
  const leftComponents = parseVersionComponents(left);
  const rightComponents = parseVersionComponents(right);

  if (leftComponents === null || rightComponents === null) {
    return null;
  }

  const componentCount = Math.max(leftComponents.length, rightComponents.length);
  for (let position = 0; position < componentCount; position += 1) {
    const difference = (leftComponents[position] ?? 0) - (rightComponents[position] ?? 0);
    if (difference !== 0) {
      return difference;
    }
  }

  return 0;
}
