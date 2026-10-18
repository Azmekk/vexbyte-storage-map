export type InlinePart = {
  kind: "text" | "code";
  text: string;
};

/** Splits a changelog bullet into plain text and `code` spans. An unmatched backtick stays text. */
export function splitInlineCode(text: string): InlinePart[] {
  const parts: InlinePart[] = [];
  const pattern = /`([^`]+)`/g;
  let textStart = 0;
  for (const match of text.matchAll(pattern)) {
    const matchStart = match.index;
    if (matchStart > textStart) {
      parts.push({ kind: "text", text: text.slice(textStart, matchStart) });
    }
    parts.push({ kind: "code", text: match[1] ?? "" });
    textStart = matchStart + match[0].length;
  }
  if (textStart < text.length) parts.push({ kind: "text", text: text.slice(textStart) });
  return parts;
}
