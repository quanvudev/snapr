import { createElement, useCallback, useEffect, useRef, useState } from "react";
import { toast } from "@/components/ui/toast";
import { useSettingsContext } from "@/components/settings-provider";
import { QueueToast } from "@/components/settings/gallery/queue-toast";
import { api, type QueueItem } from "@/lib/api";
import { taskError } from "@/lib/queue";

export function useQueue() {
  const { t } = useSettingsContext();
  const strings = useRef(t);
  strings.current = t;
  const [items, setItems] = useState<QueueItem[]>([]);
  const itemsRef = useRef<QueueItem[]>([]);
  const finished = useRef(0);
  const succeeded = useRef(0);
  const toastId = useRef<string | null>(null);
  const dismissed = useRef(false);

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
    finished.current = 0;
    succeeded.current = 0;
  }, []);

  useEffect(() => {
    const load = () => api.queueList().then(setItems).catch(report);
    load();
    const subs = [
      api.onQueue(setItems),
      api.onSettingsOpen(load),
      api.onQueueEnded((ended) => {
        finished.current += 1;
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
  }, [report, settle]);

  useEffect(() => {
    itemsRef.current = items;
    if (!items.length) {
      dismissed.current = false;
      if (toastId.current) toast.close(toastId.current);
      toastId.current = null;
      settle();
      return;
    }
    if (dismissed.current) return;
    const title = t.queueProcessing
      .replace("{index}", String(finished.current + 1))
      .replace("{total}", String(finished.current + items.length));
    const description = createElement(QueueToast, {
      t,
      items,
      onCancel: (id: number) => api.queueCancel(id).catch(report),
      onCancelAll: () => api.queueCancelAll().catch(report),
    });
    if (toastId.current) {
      toast.update(toastId.current, { title, description });
      return;
    }
    toastId.current = toast.add({
      type: "loading",
      timeout: 0,
      title,
      description,
      onClose: () => {
        toastId.current = null;
        dismissed.current = itemsRef.current.length > 0;
      },
    });
  }, [items, t, report, settle]);
}
