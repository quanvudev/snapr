import { useCallback, useEffect, useRef, useState } from "react";
import { useSettingsContext } from "@/components/settings-provider";
import { useTaskError } from "./use-task-error";
import { api, type GalleryItem, type QueueItem, type Segment, type Track } from "@/lib/api";
import { MS_PER_SECOND, percent, taskError, trackDetail, trackKind, trackSpec } from "@/lib/queue";

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

const trackKey = (path: string, track: Track) => `${track}:${path}`;

export function useTranscript(item: GalleryItem, track: Track, video: HTMLVideoElement | null) {
  const { t } = useSettingsContext();
  const strings = useRef(t);
  strings.current = t;
  const path = item.path;
  const key = trackKey(path, track);
  const kind = trackKind(track);
  const detail = trackDetail(track);
  const [loaded, setLoaded] = useState<{ key: string; segments: Segment[] | null }>();
  const [job, setJob] = useState<QueueItem | null>(null);
  const [errors, setErrors] = useState<Map<string, string>>(new Map());
  const [active, setActive] = useState(-1);
  const unknownLanguage = item.language == null;

  const reportTask = useTaskError();
  const report = useCallback((e: unknown) => reportTask(e), [reportTask]);

  const clearError = useCallback(
    () =>
      setErrors((prev) => {
        const next = new Map(prev);
        next.delete(key);
        return next;
      }),
    [key],
  );

  const load = useCallback(() => {
    api
      .transcriptRead(path, track)
      .then((segments) => setLoaded({ key, segments }))
      .catch(report);
  }, [path, track, key, report]);

  useEffect(() => {
    load();
  }, [load]);

  useEffect(() => {
    if (unknownLanguage) api.transcriptDetect(path).catch(() => {});
  }, [path, unknownLanguage]);

  useEffect(() => {
    const matches = (i: { kind: string; source: string | null; detail: string | null }) =>
      i.kind === kind && i.source === path && (detail === null || i.detail === detail);
    const find = (items: QueueItem[]) => items.find(matches) ?? null;
    api.queueList().then((items) => setJob(find(items))).catch(report);
    const subs = [
      api.onQueue((items) => setJob(find(items))),
      api.onQueueEnded((ended) => {
        if (!matches(ended)) return;
        if (ended.error) {
          const message = taskError(strings.current, ended.error);
          setErrors((prev) => new Map(prev).set(key, message));
        } else {
          clearError();
          load();
        }
      }),
    ];
    return () => {
      subs.forEach((s) => s.then((f) => f()));
    };
  }, [path, kind, detail, key, load, clearError, report]);

  const segments = loaded?.key === key ? loaded.segments : undefined;

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
    clearError();
    api.queueAdd(trackSpec(path, track)).catch(report);
  };

  const save = (index: number, text: string) => {
    if (!segments) return Promise.resolve();
    const next = segments.map((segment, i) => (i === index ? { ...segment, text } : segment));
    return api
      .transcriptSave(path, track, next)
      .then(() => setLoaded({ key, segments: next }))
      .catch(report);
  };

  return {
    segments,
    active,
    running: job ? { waiting: job.state === "waiting", percent: percent(job.done, job.total) } : null,
    error: errors.get(key) ?? null,
    create,
    save,
    cancel: () => {
      if (job) api.queueCancel(job.id).catch(report);
    },
    seek: (ms: number) => {
      if (video) video.currentTime = ms / MS_PER_SECOND;
    },
  };
}

export function useTranslationJobs(path: string) {
  const [targets, setTargets] = useState<string[]>([]);
  useEffect(() => {
    const pick = (items: QueueItem[]) =>
      setTargets(items.filter((i) => i.kind === "translate" && i.source === path && i.detail).map((i) => i.detail!));
    api.queueList().then(pick).catch(() => {});
    const sub = api.onQueue(pick);
    return () => {
      sub.then((f) => f());
    };
  }, [path]);
  return targets;
}
