import { useState } from "react";
import { X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Item, ItemContent, ItemDescription, ItemFooter, ItemTitle } from "@/components/ui/item";
import { useSettingsContext } from "@/components/settings-provider";
import { api } from "@/lib/api";

const MAX_PATTERNS = 20;

type Props = { patterns: string[]; onChange: (patterns: string[]) => void };

export function RedactPatterns({ patterns, onChange }: Props) {
  const { t } = useSettingsContext();
  const [draft, setDraft] = useState("");
  const [error, setError] = useState<string | null>(null);
  const full = patterns.length >= MAX_PATTERNS;

  const add = async () => {
    if (patterns.includes(draft)) {
      setError(t.redactDuplicate);
      return;
    }
    try {
      await api.redactValidate(draft);
    } catch (e) {
      const message = String(e);
      setError(
        message === "EMPTY"
          ? t.redactEmpty
          : message === "TOO_LONG"
            ? t.redactTooLong
            : t.redactInvalid.replace("{error}", message),
      );
      return;
    }
    setError(null);
    setDraft("");
    onChange([...patterns, draft]);
  };

  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.redactPatterns}</ItemTitle>
        <ItemDescription>{t.redactPatternsHint}</ItemDescription>
      </ItemContent>
      <ItemFooter className="flex-col items-stretch gap-2">
        {patterns.map((pattern) => (
          <div key={pattern} className="flex items-center gap-2 rounded-md border bg-background/50 py-1 pr-1 pl-3">
            <code className="flex-1 truncate font-mono text-xs">{pattern}</code>
            <Button
              variant="ghost"
              size="icon-sm"
              title={t.redactRemove}
              onClick={() => onChange(patterns.filter((p) => p !== pattern))}
            >
              <X />
            </Button>
          </div>
        ))}
        <form
          className="flex gap-2"
          onSubmit={(e) => {
            e.preventDefault();
            if (!full) void add();
          }}
        >
          <Input
            value={draft}
            placeholder={t.redactPlaceholder}
            className="font-mono text-xs"
            aria-invalid={!!error}
            onChange={(e) => {
              setDraft(e.target.value);
              setError(null);
            }}
          />
          <Button type="submit" variant="outline" disabled={full}>
            {t.redactAdd}
          </Button>
        </form>
        {error && <p className="text-sm text-destructive">{error}</p>}
        {full && <p className="text-sm text-muted-foreground">{t.redactLimit.replace("{n}", String(MAX_PATTERNS))}</p>}
      </ItemFooter>
    </Item>
  );
}
