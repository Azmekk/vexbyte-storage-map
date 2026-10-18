import { splitInlineCode } from "../lib/changelog/splitInlineCode";

export function InlineCodeText({ text }: { text: string }) {
  return (
    <>
      {splitInlineCode(text).map((part, index) =>
        part.kind === "code" ? (
          // biome-ignore lint/suspicious/noArrayIndexKey: the parts of one fixed string never reorder.
          <code key={index}>{part.text}</code>
        ) : (
          // biome-ignore lint/suspicious/noArrayIndexKey: the parts of one fixed string never reorder.
          <span key={index}>{part.text}</span>
        ),
      )}
    </>
  );
}
