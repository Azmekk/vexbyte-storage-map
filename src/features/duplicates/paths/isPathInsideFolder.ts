import { currentPlatform } from "@/lib/platform/currentPlatform";

type PathComparisonRules = {
  separator: string;
  ignoresCase: boolean;
};

function currentPathComparisonRules(): PathComparisonRules {
  const platform = currentPlatform();
  return {
    separator: platform.os === "windows" ? "\\" : "/",
    ignoresCase: !platform.pathsAreCaseSensitive,
  };
}

/** The path with one separator style and no trailing separator; the root `/` becomes empty. */
function normalizeForComparison(path: string, rules: PathComparisonRules): string {
  const unifiedSeparators = rules.separator === "\\" ? path.replaceAll("/", "\\") : path;
  const withoutTrailingSeparators = unifiedSeparators.replace(
    rules.separator === "\\" ? /\\+$/ : /\/+$/,
    "",
  );
  return rules.ignoresCase ? withoutTrailingSeparators.toLowerCase() : withoutTrailingSeparators;
}

/**
 * Whether `path` is `folder` itself or lies anywhere below it. Ignores case where the system
 * does, like Windows; on Linux the case must match.
 */
export function isPathInsideFolder(path: string, folder: string): boolean {
  if (folder.trim().length === 0) {
    return false;
  }

  const rules = currentPathComparisonRules();
  const normalizedPath = normalizeForComparison(path, rules);
  const normalizedFolder = normalizeForComparison(folder, rules);

  if (normalizedFolder.length === 0) {
    return path.startsWith(rules.separator);
  }

  return (
    normalizedPath === normalizedFolder ||
    normalizedPath.startsWith(`${normalizedFolder}${rules.separator}`)
  );
}

/** Whether two folder paths name the same folder, ignoring trailing separators. */
export function isSameFolder(firstFolder: string, secondFolder: string): boolean {
  const rules = currentPathComparisonRules();
  return normalizeForComparison(firstFolder, rules) === normalizeForComparison(secondFolder, rules);
}
