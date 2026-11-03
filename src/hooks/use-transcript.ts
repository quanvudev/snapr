import { useCallback, useEffect, useRef, useState } from "react";
import { toast } from "@/components/ui/toast";
import { useSettingsContext } from "@/components/settings-provider";
import { api, type QueueItem, type Segment } from "@/lib/api";
import { MS_PER_SECOND, percent, taskError, transcribeSpec } from "@/lib/queue";

export function activeSegment(segments: Segment[], ms: number) {
  let low = 0;
  let high = segments.length - 1;
  let found = -1;
  while (low <= high) {
    const mid = Math.floor((low + high) / 2);
    if (segments[mid].start_ms <= ms) {
      found = mid;
      low = mid + 1;
    } else {
      high = mid - 1;
    }
  }
  return found;
}

export function useTranscript(path: string, video: HTMLVideoElement | null) {
  const { t } = useSettingsContext();
  const strings = useRef(t);
  strings.current = t;
  const [loaded, setLoaded] = useState<{ path: string; segments: Segment[] | null }>();
  const [job, setJob] = useState<QueueItem | null>(null);
  const [errors, setErrors] = useState<Map<string, string>>(new Map());
  const [active, setActive] = useState(-1);

  const report = useCallback(
    (e: unknown) =>
      toast.add({ type: "error", title: strings.current.galleryFailed, description: taskError(strings.current, e) }),
    [],
  );

  const load = useCallback(() => {
    api
      .transcriptRead(path)
      .then((segments) => setLoaded({ path, segments }))
      .catch(report);
  }, [path, report]);

  useEffect(() => {
    load();
  }, [load]);

  useEffect(() => {
    const find = (items: QueueItem[]) => items.find((item) => item.source === path) ?? null;
    api.queueList().then((items) => setJob(find(items))).catch(report);
    const subs = [
      api.onQueue((items) => setJob(find(items))),
      api.onQueueEnded((ended) => {
        if (ended.source !== path) return;
        if (ended.error) {
          const message = taskError(strings.current, ended.error);
          setErrors((prev) => new Map(prev).set(path, message));
        } else {
          setErrors((prev) => {
            const next = new Map(prev);
            next.delete(path);
            return next;
          });
          load();
        }
      }),
    ];
    return () => {
      subs.forEach((s) => s.then((f) => f()));
    };
  }, [path, load, report]);

  const segments = loaded?.path === path ? loaded.segments : undefined;

  useEffect(() => {
    if (!video || !segments) return;
    const sync = () => setActive(activeSegment(segments, video.currentTime * MS_PER_SECOND));
    sync();
    video.addEventListener("timeupdate", sync);
    video.addEventListener("seeked", sync);
    return () => {
      video.removeEventListener("timeupdate", sync);
      video.removeEventListener("seeked", sync);
    };
  }, [video, segments]);

  const create = () => {
    setErrors((prev) => {
      const next = new Map(prev);
      next.delete(path);
      return next;
    });
    api.queueAdd(transcribeSpec(path)).catch(report);
  };

  return {
    segments,
    active,
    running: job ? { waiting: job.state === "waiting", percent: percent(job.done, job.total) } : null,
    error: errors.get(path) ?? null,
    create,
    cancel: () => {
      if (job) api.queueCancel(job.id).catch(report);
    },
    seek: (ms: number) => {
      if (video) video.currentTime = ms / MS_PER_SECOND;
    },
  };
}
