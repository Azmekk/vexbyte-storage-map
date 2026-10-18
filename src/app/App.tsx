import { MotionConfig } from "motion/react";
import { Toaster } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { UpdatePrompts } from "@/features/updates/UpdatePrompts";
import { AppLayout } from "./AppLayout";
import { ThemeProvider } from "./ThemeProvider";

export function App() {
  return (
    <ThemeProvider>
      <MotionConfig reducedMotion="user">
        <TooltipProvider delayDuration={300}>
          <AppLayout />
          <Toaster />
          <UpdatePrompts />
        </TooltipProvider>
      </MotionConfig>
    </ThemeProvider>
  );
}
