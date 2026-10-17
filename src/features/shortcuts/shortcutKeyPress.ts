/** The parts of a key press the shortcut mapping looks at. */
export type ShortcutKeyPress = {
  key: string;
  ctrlKey: boolean;
  metaKey: boolean;
  altKey: boolean;
  /** The focus is in a text field, where keys such as Backspace must keep their usual meaning. */
  isTyping: boolean;
  /** A dialog, menu or list popup is open and owns the keyboard. */
  isOverlayOpen: boolean;
  /** Another handler (for example the tree table) already handled the key. */
  isHandled: boolean;
};

const OPEN_OVERLAY_SELECTOR =
  '[role="dialog"], [role="alertdialog"], [role="menu"], [role="listbox"]';

function isTypingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false;
  }
  return (
    target.isContentEditable ||
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement
  );
}

export function readShortcutKeyPress(event: KeyboardEvent): ShortcutKeyPress {
  return {
    key: event.key,
    ctrlKey: event.ctrlKey,
    metaKey: event.metaKey,
    altKey: event.altKey,
    isTyping: isTypingTarget(event.target),
    isOverlayOpen: document.querySelector(OPEN_OVERLAY_SELECTOR) !== null,
    isHandled: event.defaultPrevented,
  };
}
