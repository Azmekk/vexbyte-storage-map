import type { KeepRule } from "@/lib/backend/generated";

export type KeepRuleKind = KeepRule["kind"];

export type KeepRuleOption = {
  kind: KeepRuleKind;
  label: string;
  description: string;
};

export const KEEP_RULE_OPTIONS: readonly KeepRuleOption[] = [
  {
    kind: "oldest",
    label: "Keep the oldest",
    description: "The copy modified longest ago, usually the original.",
  },
  {
    kind: "newest",
    label: "Keep the newest",
    description: "The copy modified most recently.",
  },
  {
    kind: "preferredFolder",
    label: "Keep the one in a folder…",
    description: "The copy inside a folder you pick. Groups without one keep their oldest copy.",
  },
  {
    kind: "shortestPath",
    label: "Keep the shortest path",
    description: "The copy in the tidiest place, with the shortest full path.",
  },
];

export const DEFAULT_KEEP_RULE: KeepRule = { kind: "oldest" };

export function isKeepRuleKind(value: string): value is KeepRuleKind {
  return KEEP_RULE_OPTIONS.some((option) => option.kind === value);
}

/**
 * The rule a selector choice stands for. A preferred folder rule needs a folder; without one the
 * result is `null` and the caller must ask for a folder first.
 */
export function keepRuleFromKind(
  kind: KeepRuleKind,
  preferredFolder: string | null,
): KeepRule | null {
  switch (kind) {
    case "oldest":
      return { kind: "oldest" };
    case "newest":
      return { kind: "newest" };
    case "shortestPath":
      return { kind: "shortestPath" };
    case "preferredFolder":
      return preferredFolder === null ? null : { kind: "preferredFolder", folder: preferredFolder };
  }
}

export function keepRuleOptionFor(rule: KeepRule): KeepRuleOption {
  const matchingOption = KEEP_RULE_OPTIONS.find((option) => option.kind === rule.kind);
  return matchingOption ?? KEEP_RULE_OPTIONS[0] ?? { kind: "oldest", label: "", description: "" };
}

export function preferredFolderOf(rule: KeepRule): string | null {
  return rule.kind === "preferredFolder" ? rule.folder : null;
}
