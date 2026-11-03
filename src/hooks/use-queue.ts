import { useCallback, useEffect, useRef, useState } from "react";
import { toast } from "@/components/ui/toast";
import { useSettingsContext } from "@/components/settings-provider";
import { api, type HistoryItem, type QueueItem } from "@/lib/api";
import { taskError } from "@/lib/queue";

export function useQueue() {
  const { t } = useSettingsContext();
  const strings = useRef(t);
  strings.current = t;
  const [items, setItems] = useState<QueueItem[]>([]);
  const [history, setHistory] = useState<HistoryItem[]>([]);
  const [open, setOpen] = useState(false);
  const itemsRef = useRef<QueueItem[]>([]);
  const knownIds = useRef<Set<number>>(new Set());
  const succeeded = useRef(0);

  const report = useCallback(
    (e: unknown) => toast.add({ type: "error", title: strings.current.galleryFailed, description: String(e) }),
    [],
  );

  const settle = useCallback(() => {
    if (succeeded.current > 0) {
      toast.add({
        type: "success",
        title: strings.current.queueDone.replace("{count}", String(succeeded.current)),
      });
    }
    succeeded.current = 0;
  }, []);

  const announceAdded = useCallback((added: QueueItem[]) => {
    const text = strings.current;
    const title =
      added.length === 1
        ? text.queueAdded.replace("{name}", added[0].name)
        : text.queueAddedMany.replace("{count}", String(added.length));
    toast.add({ title, actionProps: { children: text.queueView, onClick: () => setOpen(true) } });
  }, []);

  const follow = useCallback(
    (list: QueueItem[]) => {
      const added = list.filter((item) => !knownIds.current.has(item.id));
      knownIds.current = new Set(list.map((item) => item.id));
      setItems(list);
      if (added.length) announceAdded(added);
    },
    [announceAdded],
  );

  useEffect(() => {
    const load = () => {
      api
        .queueList()
        .then((list) => {
          knownIds.current = new Set(list.map((item) => item.id));
          setItems(list);
        })
        .catch(report);
      api.queueHistory().then(setHistory).catch(report);
    };
    load();
    const subs = [
      api.onQueue(follow),
      api.onQueueHistory(setHistory),
      api.onSettingsOpen(load),
      api.onQueueEnded((ended) => {
        if (ended.error) {
          toast.add({
            type: "error",
            title: strings.current.queueFailed,
            description: `${ended.name}: ${taskError(strings.current, ended.error)}`,
          });
          return;
        }
        succeeded.current += 1;
        if (!itemsRef.current.length) settle();
      }),
    ];
    return () => {
      subs.forEach((s) => s.then((f) => f()));
    };
  }, [report, settle, follow]);

  useEffect(() => {
    itemsRef.current = items;
    if (!items.length) settle();
  }, [items, settle]);

  return {
    items,
    history,
    count: items.length,
    open,
    setOpen,
    cancel: (id: number) => api.queueCancel(id).catch(report),
    cancelAll: () => api.queueCancelAll().catch(report),
    retry: (id: number) =>
      api.queueRetry(id).catch((e) =>
        toast.add({ type: "error", title: strings.current.queueFailed, description: taskError(strings.current, e) }),
      ),
    reveal: (path: string) => api.galleryReveal(path).catch(report),
    clear: () => api.queueHistoryClear().catch(report),
  };
}
