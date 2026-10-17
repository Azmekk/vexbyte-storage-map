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
import { folderDisplayName } from "../paths/folderDisplayName";
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
        <div className="flex h-9 max-w-72 items-center gap-1 rounded-md border bg-card pr-1 pl-2.5 text-sm">
          <FolderIcon className="size-4 shrink-0 text-muted-foreground" aria-hidden />
          <Tooltip>
            <TooltipTrigger asChild>
              <span className="min-w-0 truncate">{folderDisplayName(preferredFolder)}</span>
            </TooltipTrigger>
            <TooltipContent>{preferredFolder}</TooltipContent>
          </Tooltip>
          <Button
            variant="ghost"
            size="sm"
            className="h-7 px-2 text-muted-foreground"
            onClick={onChoosePreferredFolder}
            disabled={isApplying}
            aria-label={`Change the folder whose copies to keep (now ${preferredFolder})`}
          >
            Change
          </Button>
        </div>
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
