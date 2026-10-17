/** `1 file`, `3 files`, `1,204 copies`: the count with thousands separators and the right noun. */
export function pluralize(count: number, singularNoun: string, pluralNoun: string): string {
  const noun = Math.round(count) === 1 ? singularNoun : pluralNoun;
  return `${new Intl.NumberFormat("en-US").format(Math.round(count))} ${noun}`;
}
