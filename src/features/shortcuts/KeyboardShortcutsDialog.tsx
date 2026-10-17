import { Fragment } from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Kbd, KbdGroup } from "@/components/ui/kbd";
import { KEYBOARD_SHORTCUT_GROUPS } from "./keyboardShortcutList";
import { useKeyboardShortcutsDialogStore } from "./useKeyboardShortcutsDialogStore";

export function KeyboardShortcutsDialog() {
  const isOpen = useKeyboardShortcutsDialogStore((state) => state.isOpen);
  const setOpen = useKeyboardShortcutsDialogStore((state) => state.setOpen);

  return (
    <Dialog open={isOpen} onOpenChange={setOpen}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>Keyboard shortcuts</DialogTitle>
          <DialogDescription>Get around without the mouse.</DialogDescription>
        </DialogHeader>
        <div className="grid gap-5">
          {KEYBOARD_SHORTCUT_GROUPS.map((group) => (
            <section key={group.title} className="grid gap-2">
              <h3 className="font-medium text-muted-foreground text-xs uppercase tracking-wide">
                {group.title}
              </h3>
              <dl className="grid gap-1.5">
                {group.shortcuts.map((shortcut) => (
                  <div
                    key={shortcut.description}
                    className="flex items-center justify-between gap-4 text-sm"
                  >
                    <dt>{shortcut.description}</dt>
                    <dd className="flex shrink-0 items-center gap-1.5 text-muted-foreground text-xs">
                      {shortcut.keyCombinations.map((keys, combinationIndex) => (
                        <Fragment key={keys.join("+")}>
                          {combinationIndex > 0 && <span>or</span>}
                          <KbdGroup>
                            {keys.map((key) => (
                              <Kbd key={key}>{key}</Kbd>
                            ))}
                          </KbdGroup>
                        </Fragment>
                      ))}
                    </dd>
                  </div>
                ))}
              </dl>
            </section>
          ))}
        </div>
      </DialogContent>
    </Dialog>
  );
}
