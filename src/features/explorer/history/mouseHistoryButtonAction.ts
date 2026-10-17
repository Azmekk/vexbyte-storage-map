/** `MouseEvent.button` of the side buttons most mice have. */
const MOUSE_BACK_BUTTON = 3;
const MOUSE_FORWARD_BUTTON = 4;

/** What a mouse button does to the folder history: the side buttons go back and forward. */
export function mouseHistoryButtonAction(button: number): "goBack" | "goForward" | null {
  if (button === MOUSE_BACK_BUTTON) {
    return "goBack";
  }
  if (button === MOUSE_FORWARD_BUTTON) {
    return "goForward";
  }
  return null;
}
