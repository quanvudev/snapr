import { Check, CircleAlert, FolderOpen, RotateCcw, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Empty, EmptyHeader, EmptyTitle } from "@/components/ui/empty";
import { Progress } from "@/components/ui/progress";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Sheet, SheetContent, SheetHeader, SheetTitle } from "@/components/ui/sheet";
import type { HistoryItem, QueueItem } from "@/lib/api";
import { percent, taskError, TASK_KINDS, type Strings } from "@/lib/queue";

type Props = {
  t: Strings;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  items: QueueItem[];
  history: HistoryItem[];
  onCancel: (id: number) => void;
  onCancelAll: () => void;
  onRetry: (id: number) => void;
  onReveal: (path: string) => void;
  onClear: () => void;
};

const kindLabel = (t: Strings, kind: string) => t[TASK_KINDS[kind]?.label ?? "queueWaiting"];

export function QueueSheet({ t, open, onOpenChange, items, history, onCancel, onCancelAll, onRetry, onReveal, onClear }: Props) {
  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent className="sm:max-w-md">
        <SheetHeader>
          <SheetTitle>{t.queueTitle}</SheetTitle>
        </SheetHeader>
        <ScrollArea className="min-h-0 flex-1">
          <div className="flex flex-col gap-4 px-4 pb-4">
            {items.length > 0 && (
              <section className="flex flex-col gap-2">
                <div className="flex items-center gap-2">
                  <span className="min-w-0 flex-1 text-xs font-medium text-muted-foreground">{t.queueActive}</span>
                  <Button variant="outline" size="xs" onClick={onCancelAll}>
                    {t.queueCancelAll}
                  </Button>
                </div>
                {items.map((item) => (
                  <ActiveRow key={item.id} t={t} item={item} onCancel={onCancel} />
                ))}
              </section>
            )}
            {history.length > 0 && (
              <section className="flex flex-col gap-2">
                <div className="flex items-center gap-2">
                  <span className="min-w-0 flex-1 text-xs font-medium text-muted-foreground">{t.queueFinished}</span>
                  <Button variant="outline" size="xs" onClick={onClear}>
                    {t.queueClear}
                  </Button>
                </div>
                {history.map((entry) => (
                  <HistoryRow key={entry.id} t={t} entry={entry} onRetry={onRetry} onReveal={onReveal} />
                ))}
              </section>
            )}
            {!items.length && !history.length && (
              <Empty>
                <EmptyHeader>
                  <EmptyTitle>{t.queueEmpty}</EmptyTitle>
                </EmptyHeader>
              </Empty>
            )}
          </div>
        </ScrollArea>
      </SheetContent>
    </Sheet>
  );
}

function ActiveRow({ t, item, onCancel }: { t: Strings; item: QueueItem; onCancel: (id: number) => void }) {
  const running = item.state === "running";
  return (
    <div className="flex flex-col gap-1">
      <div className="flex items-center gap-2">
        <span className="min-w-0 flex-1 truncate">
          {item.name} · {kindLabel(t, item.kind)}
        </span>
        <span className="shrink-0 text-xs tabular-nums">
          {running ? `${Math.round(percent(item.done, item.total))}%` : t.queueWaiting}
        </span>
        <Button variant="ghost" size="icon-xs" title={t.queueCancel} onClick={() => onCancel(item.id)}>
          <X />
        </Button>
      </div>
      {running && <Progress value={percent(item.done, item.total)} />}
    </div>
  );
}

function HistoryRow({
  t,
  entry,
  onRetry,
  onReveal,
}: {
  t: Strings;
  entry: HistoryItem;
  onRetry: (id: number) => void;
  onReveal: (path: string) => void;
}) {
  return (
    <div className="flex items-center gap-2">
      <div className="flex min-w-0 flex-1 flex-col">
        <span className="truncate">
          {entry.name} · {kindLabel(t, entry.kind)}
        </span>
        {entry.state === "failed" && entry.error && (
          <span className="truncate text-xs text-destructive" title={taskError(t, entry.error)}>
            {taskError(t, entry.error)}
          </span>
        )}
      </div>
      {entry.state === "done" && <Check className="size-4 shrink-0" />}
      {entry.state === "failed" && <CircleAlert className="size-4 shrink-0 text-destructive" />}
      {entry.state === "cancelled" && <span className="shrink-0 text-xs">{t.queueCancelled}</span>}
      {entry.state === "done" && entry.output && (
        <Button variant="ghost" size="icon-xs" title={t.showInExplorer} onClick={() => onReveal(entry.output!)}>
          <FolderOpen />
        </Button>
      )}
      {entry.state !== "done" && entry.retryable && (
        <Button variant="ghost" size="icon-xs" title={t.paddleRetry} onClick={() => onRetry(entry.id)}>
          <RotateCcw />
        </Button>
      )}
    </div>
  );
}
