import { CircleHelpIcon } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { SEARCH_SYNTAX_NOTES, SEARCH_SYNTAX_SECTIONS } from "./searchSyntaxHelp";

type SearchSyntaxHelpPopoverProps = {
  onUseExample: (example: string) => void;
};

export function SearchSyntaxHelpPopover({ onUseExample }: SearchSyntaxHelpPopoverProps) {
  const [isOpen, setOpen] = useState(false);

  return (
    <Popover open={isOpen} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button variant="ghost" size="sm" className="text-muted-foreground">
          <CircleHelpIcon />
          Search syntax
        </Button>
      </PopoverTrigger>
      <PopoverContent align="end" className="w-[min(34rem,calc(100vw-2rem))] gap-4 p-4">
        <div className="grid gap-1">
          <h2 className="font-medium text-sm">Search syntax</h2>
          <p className="text-muted-foreground text-xs">
            Click an example to try it. Matching ignores upper and lower case.
          </p>
        </div>
        {SEARCH_SYNTAX_SECTIONS.map((section) => (
          <section key={section.title} className="grid gap-1.5">
            <h3 className="text-muted-foreground text-xs">{section.title}</h3>
            <dl className="grid grid-cols-[minmax(9rem,auto)_1fr] gap-x-4 gap-y-1">
              {section.entries.map((entry) => (
                <div key={entry.example} className="contents">
                  <dt>
                    <button
                      type="button"
                      onClick={() => {
                        onUseExample(entry.example);
                        setOpen(false);
                      }}
                      className="rounded-sm bg-muted px-1.5 py-0.5 text-left font-mono text-xs outline-none hover:bg-accent focus-visible:ring-2 focus-visible:ring-ring"
                    >
                      {entry.example}
                    </button>
                  </dt>
                  <dd className="self-center text-muted-foreground text-xs">{entry.description}</dd>
                </div>
              ))}
            </dl>
          </section>
        ))}
        <ul className="grid list-disc gap-0.5 border-t pt-3 pl-4 text-muted-foreground text-xs">
          {SEARCH_SYNTAX_NOTES.map((note) => (
            <li key={note}>{note}</li>
          ))}
        </ul>
      </PopoverContent>
    </Popover>
  );
}
