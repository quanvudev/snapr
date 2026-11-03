import { useState } from "react";
import { ChevronDown, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import { Progress } from "@/components/ui/progress";
import type { QueueItem } from "@/lib/api";
import { percent, TASK_KINDS, type Strings } from "@/lib/queue";

type Props = {
  t: Strings;
  items: QueueItem[];
  onCancel: (id: number) => void;
  onCancelAll: () => void;
};

export function QueueToast({ t, items, onCancel, onCancelAll }: Props) {
  const [open, setOpen] = useState(false);
  const running = items.find((item) => item.state === "running") ?? items[0];
  if (!running) return null;
  return (
    <Collapsible open={open} onOpenChange={setOpen} className="flex flex-col gap-2">
      <div className="flex items-center gap-2">
        <span className="min-w-0 flex-1 truncate">{running.name}</span>
        <Button variant="outline" size="xs" onClick={onCancelAll}>
          {t.queueCancelAll}
        </Button>
        <CollapsibleTrigger
          render={<Button variant="ghost" size="icon-sm" title={open ? t.queueHide : t.queueShow} />}
        >
          <ChevronDown className={open ? "rotate-180" : undefined} />
        </CollapsibleTrigger>
      </div>
      <Progress value={percent(running.done, running.total)} />
      <CollapsibleContent className="flex flex-col gap-1">
        {items.map((item) => (
          <div key={item.id} className="flex items-center gap-2">
            <span className="min-w-0 flex-1 truncate">
              {item.name} · {t[TASK_KINDS[item.kind]?.label ?? "queueWaiting"]}
            </span>
            <span className="shrink-0 text-xs tabular-nums">
              {item.state === "running" ? `${Math.round(percent(item.done, item.total))}%` : t.queueWaiting}
            </span>
            <Button variant="ghost" size="icon-xs" title={t.queueCancel} onClick={() => onCancel(item.id)}>
              <X />
            </Button>
          </div>
        ))}
      </CollapsibleContent>
    </Collapsible>
  );
}
