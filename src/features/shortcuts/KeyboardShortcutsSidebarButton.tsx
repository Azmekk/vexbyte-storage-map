import { KeyboardIcon } from "lucide-react";
import { Kbd } from "@/components/ui/kbd";
import { SidebarMenu, SidebarMenuButton, SidebarMenuItem } from "@/components/ui/sidebar";
import { useKeyboardShortcutsDialogStore } from "./useKeyboardShortcutsDialogStore";

export function KeyboardShortcutsSidebarButton() {
  const setOpen = useKeyboardShortcutsDialogStore((state) => state.setOpen);

  return (
    <SidebarMenu>
      <SidebarMenuItem>
        <SidebarMenuButton
          tooltip="Keyboard shortcuts (?)"
          onClick={() => setOpen(true)}
          className="text-sidebar-foreground/70"
        >
          <KeyboardIcon />
          <span>Keyboard shortcuts</span>
          <Kbd className="ml-auto group-data-[collapsible=icon]:hidden">?</Kbd>
        </SidebarMenuButton>
      </SidebarMenuItem>
    </SidebarMenu>
  );
}
