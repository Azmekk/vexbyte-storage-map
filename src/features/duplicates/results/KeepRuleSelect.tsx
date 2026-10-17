import { FolderIcon, LoaderCircleIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import type { KeepRule } from "@/lib/backend/generated";
import { splitFilePath } from "../paths/splitFilePath";
import {
  isKeepRuleKind,
  KEEP_RULE_OPTIONS,
  type KeepRuleKind,
  keepRuleOptionFor,
  preferredFolderOf,
} from "./keepRuleOptions";

type KeepRuleSelectProps = {
  keepRule: KeepRule;
  isApplying: boolean;
  onChooseKeepRule: (kind: KeepRuleKind) => void;
  onChoosePreferredFolder: () => void;
};

export function KeepRuleSelect({
  keepRule,
  isApplying,
  onChooseKeepRule,
  onChoosePreferredFolder,
}: KeepRuleSelectProps) {
  const selectedOption = keepRuleOptionFor(keepRule);
  const preferredFolder = preferredFolderOf(keepRule);

  return (
    <div className="flex flex-wrap items-center gap-2">
      <Label htmlFor="duplicate-keep-rule" className="text-muted-foreground">
        For every group
      </Label>
      <Select
        value={keepRule.kind}
        onValueChange={(value) => {
          if (isKeepRuleKind(value)) {
            onChooseKeepRule(value);
          }
        }}
        disabled={isApplying}
      >
        <Tooltip>
          <TooltipTrigger asChild>
            <SelectTrigger id="duplicate-keep-rule" className="min-w-48">
              <SelectValue />
            </SelectTrigger>
          </TooltipTrigger>
          <TooltipContent>{selectedOption.description}</TooltipContent>
        </Tooltip>
        <SelectContent>
          {KEEP_RULE_OPTIONS.map((option) => (
            <SelectItem key={option.kind} value={option.kind}>
              {option.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      {preferredFolder !== null && (
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="outline"
              onClick={onChoosePreferredFolder}
              disabled={isApplying}
              className="max-w-56"
            >
              <FolderIcon />
              <span className="truncate">
                {splitFilePath(preferredFolder.replace(/[\\/]+$/, "")).fileName || preferredFolder}
              </span>
            </Button>
          </TooltipTrigger>
          <TooltipContent>{preferredFolder} (click to change)</TooltipContent>
        </Tooltip>
      )}
      {isApplying && (
        <LoaderCircleIcon
          className="size-4 animate-spin text-muted-foreground"
          aria-label="Applying"
        />
      )}
    </div>
  );
}
