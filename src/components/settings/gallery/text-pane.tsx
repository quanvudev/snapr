import { useState } from "react";
import { Check, Copy, X } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Spinner } from "@/components/ui/spinner";
import { useSettingsContext } from "@/components/settings-provider";
import type { TextResult } from "@/hooks/use-gallery";

export function TextPane({ text, onClose }: { text: TextResult; onClose: () => void }) {
  const { t } = useSettingsContext();
  const [copied, setCopied] = useState(false);
  const { result, error } = text;
  const all = result ? [...result.qr, result.text].filter(Boolean).join("\n\n") : "";

  const copy = () =>
    navigator.clipboard.writeText(all).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    });

  return (
    <aside className="flex w-80 shrink-0 flex-col rounded-lg border bg-muted/40">
      <div className="flex items-center gap-1 border-b p-2">
        <span className="flex-1 pl-1 text-sm font-medium">{t.extractedText}</span>
        <Button variant="ghost" size="icon-sm" title={t.copyAll} disabled={!all} onClick={copy}>
          {copied ? <Check /> : <Copy />}
        </Button>
        <Button variant="ghost" size="icon-sm" title={t.close} onClick={onClose}>
          <X />
        </Button>
      </div>
      <ScrollArea className="min-h-0 flex-1">
        <div className="cursor-text space-y-4 p-3 text-sm select-text">
          {error ? (
            <p className="text-destructive">{error}</p>
          ) : !result ? (
            <div className="flex justify-center py-8">
              <Spinner />
            </div>
          ) : !all ? (
            <p className="text-muted-foreground">{t.noText}</p>
          ) : (
            <>
              {result.qr.map((code, i) => (
                <div key={i} className="space-y-1">
                  <Badge variant="secondary">QR</Badge>
                  <p className="break-words whitespace-pre-wrap">{code}</p>
                </div>
              ))}
              {result.text && <p className="break-words whitespace-pre-wrap">{result.text}</p>}
            </>
          )}
        </div>
      </ScrollArea>
    </aside>
  );
}
