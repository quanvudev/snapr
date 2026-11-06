import { useCallback, useEffect, useRef, useState } from "react";
import { toast } from "@/components/ui/toast";
import { useSettingsContext } from "@/components/settings-provider";
import { useTaskError } from "./use-task-error";
import { api, type BulkFailure, type GalleryItem, type Recognized, type Track } from "@/lib/api";
import { fileName, groupBy, isMp4, isViewable, matches, type Filter, type Period } from "@/lib/gallery";
import {
  compressSpec,
  gifSpec,
  trimSpec,
  loadCompressChoice,
  loadGifPreset,
  saveCompressChoice,
  saveGifPreset,
  saveSubtitlePreset,
  subtitleSpec,
  transcribeSpec,
  cutoutSpec,
  translateSpec,
  type CompressChoice,
  type GifRange,
  type SubtitlePreset,
} from "@/lib/queue";

export type TextResult = { result?: Recognized; error?: string };

const TILE_KEY = "gallery.tile";
export const TILE = { min: 8, max: 20, initial: 12 };

function storedTile() {
  try {
    const value = Number(localStorage.getItem(TILE_KEY));
    return value >= TILE.min && value <= TILE.max ? value : TILE.initial;
  } catch {
    return TILE.initial;
  }
}

export type SubtitleTarget = { item: GalleryItem; ms: number; text: string; track: Track; tracks: Track[] };

function subtitleTracks(item: GalleryItem): Track[] {
  return [...(item.transcript ? ["original" as const] : []), ...item.translations];
}

export function useGallery() {
  const { t } = useSettingsContext();
  const title = useRef(t.galleryFailed);
  title.current = t.galleryFailed;
  const strings = useRef(t);
  strings.current = t;
  const [items, setItems] = useState<GalleryItem[]>([]);
  const [filter, setFilter] = useState<Filter>("all");
  const [period, setPeriod] = useState<Period>("day");
  const [query, setQuery] = useState("");
  const [tile, setTile] = useState(storedTile);
  const [collapsed, setCollapsed] = useState<Set<string>>(new Set());
  const [viewing, setViewing] = useState<string | null>(null);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [selectMode, setSelectMode] = useState(false);
  const [anchor, setAnchor] = useState<string | null>(null);
  const [pendingDelete, setPendingDelete] = useState<GalleryItem[] | null>(null);
  const [gifTarget, setGifTarget] = useState<GalleryItem | null>(null);
  const [trimTarget, setTrimTarget] = useState<GalleryItem | null>(null);
  const [compressTarget, setCompressTarget] = useState<GalleryItem | null>(null);
  const [translateTarget, setTranslateTarget] = useState<GalleryItem | null>(null);
  const [subtitleTarget, setSubtitleTarget] = useState<SubtitleTarget | null>(null);
  const [textOpen, setTextOpen] = useState(false);
  const [texts, setTexts] = useState<Map<string, TextResult>>(new Map());
  const [transcriptOpen, setTranscriptOpen] = useState(false);
  const [transcriptTrack, setTranscriptTrack] = useState<Track>("original");

  const reportTask = useTaskError();
  const report = useCallback(
    (e: unknown) => toast.add({ type: "error", title: title.current, description: String(e) }),
    [],
  );

  const reload = useCallback(() => {
    api
      .galleryList()
      .then((list) => {
        setItems(list);
        setSelected((prev) => {
          const paths = new Set(list.map((item) => item.path));
          const next = new Set([...prev].filter((path) => paths.has(path)));
          return next.size === prev.size ? prev : next;
        });
      })
      .catch(report);
  }, [report]);

  useEffect(() => {
    reload();
    const subs = [api.onSettingsOpen(reload), api.onGalleryChanged(reload)];
    return () => {
      subs.forEach((s) => s.then((f) => f()));
    };
  }, [reload]);

  const visible = items.filter((i) => matches(i, filter, query));
  const viewable = visible.filter(isViewable);
  const index = viewable.findIndex((i) => i.path === viewing);
  const viewer = index < 0 ? null : { item: viewable[index], index, count: viewable.length };
  const textPath = textOpen && viewer?.item.kind === "photo" ? viewer.item.path : null;
  const transcriptVideo = transcriptOpen && viewer && isMp4(viewer.item) ? viewer.item : null;

  useEffect(() => {
    if (!textPath || texts.has(textPath)) return;
    const store = (result: TextResult) => setTexts((prev) => new Map(prev).set(textPath, result));
    store({});
    api
      .galleryText(textPath)
      .then((result) => store({ result }))
      .catch((e) => store({ error: String(e) }));
  }, [textPath, texts]);

  const selectedItems = visible.filter((i) => selected.has(i.path));
  const photos = selectedItems.filter((i) => i.kind === "photo");
  const videos = selectedItems.filter((i) => isMp4(i));

  const clearSelection = () => {
    setSelected(new Set());
    setSelectMode(false);
    setAnchor(null);
  };

  const changeFilter = (next: Filter) => {
    setFilter(next);
    clearSelection();
  };

  const changePeriod = (next: Period) => {
    setPeriod(next);
    clearSelection();
  };

  const changeSelectMode = (on: boolean) => {
    setSelectMode(on);
    if (!on) {
      setSelected(new Set());
      setAnchor(null);
    }
  };

  const addSelected = (item: GalleryItem) => {
    setSelectMode(true);
    setAnchor(item.path);
    setSelected((prev) => new Set(prev).add(item.path));
  };

  const toggleSelect = (item: GalleryItem) => {
    setSelectMode(true);
    setAnchor(item.path);
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(item.path)) next.delete(item.path);
      else next.add(item.path);
      return next;
    });
  };

  const selectGroup = (group: GalleryItem[], on: boolean) => {
    setSelectMode(true);
    setSelected((prev) => {
      const next = new Set(prev);
      group.forEach((item) => (on ? next.add(item.path) : next.delete(item.path)));
      return next;
    });
  };

  const changeTile = (next: number) => {
    setTile(next);
    try {
      localStorage.setItem(TILE_KEY, String(next));
    } catch {}
  };

  const selectRange = (item: GalleryItem) => {
    const to = visible.findIndex((i) => i.path === item.path);
    const from = anchor ? visible.findIndex((i) => i.path === anchor) : -1;
    if (from < 0 || to < 0) {
      addSelected(item);
      return;
    }
    const [start, end] = from < to ? [from, to] : [to, from];
    setSelectMode(true);
    setSelected((prev) => {
      const next = new Set(prev);
      for (let i = start; i <= end; i += 1) next.add(visible[i].path);
      return next;
    });
  };

  const selectAll = () => {
    setSelectMode(true);
    setAnchor(visible[0]?.path ?? null);
    setSelected(new Set(visible.map((i) => i.path)));
  };

  const run = (p: Promise<unknown>) => {
    p.catch(report);
  };

  const step = (delta: number) => {
    const target = viewable[index + delta];
    if (target) setViewing(target.path);
  };

  const confirmDelete = async () => {
    const targets = pendingDelete;
    if (!targets?.length) return;
    setPendingDelete(null);
    let failures: BulkFailure[];
    try {
      failures = await api.galleryDeleteMany(targets.map((item) => item.path));
    } catch (e) {
      report(e);
      return;
    }
    const failed = new Set(failures.map((failure) => failure.path));
    const removed = new Set(targets.filter((item) => !failed.has(item.path)).map((item) => item.path));
    if (viewing && removed.has(viewing)) setViewing((viewable[index + 1] ?? viewable[index - 1])?.path ?? null);
    setItems((prev) => prev.filter((item) => !removed.has(item.path)));
    setSelected((prev) => {
      const next = new Set(prev);
      removed.forEach((path) => next.delete(path));
      return next;
    });
    if (failures.length) {
      report(failures.map((failure) => `${fileName(failure.path)}: ${failure.error}`).join(" · "));
    }
  };

  const exportGif = (item: GalleryItem, range: GifRange) => {
    setGifTarget(null);
    saveGifPreset(range.preset);
    run(api.queueAdd(gifSpec(item.path, range.preset, range.startMs, range.endMs)));
  };

  const trim = (item: GalleryItem, startMs: number, endMs: number) => {
    setTrimTarget(null);
    run(api.queueAdd(trimSpec(item.path, startMs, endMs)));
  };

  const compress = (item: GalleryItem, choice: CompressChoice) => {
    setCompressTarget(null);
    saveCompressChoice(choice);
    run(api.queueAdd(compressSpec(item.path, choice)));
  };

  const exportSubtitle = (item: GalleryItem, preset: SubtitlePreset, track: Track) => {
    setSubtitleTarget(null);
    saveSubtitlePreset(preset);
    api.queueAdd(subtitleSpec(item.path, preset, track)).catch((e) => reportTask(e));
  };

  const compressFailed = (e: unknown) => {
    setCompressTarget(null);
    report(e);
  };

  const compressSelected = () => {
    if (!videos.length) return;
    const choice = loadCompressChoice();
    run(
      (async () => {
        for (const item of videos) await api.queueAdd(compressSpec(item.path, choice));
      })(),
    );
  };

  const exportGifSelected = () => {
    if (!videos.length) return;
    const preset = loadGifPreset();
    run(
      (async () => {
        for (const item of videos) await api.queueAdd(gifSpec(item.path, preset, 0, null));
      })(),
    );
  };

  const transcribeAll = (targets: GalleryItem[]) => {
    if (!targets.length) return;
    (async () => {
      for (const item of targets) await api.queueAdd(transcribeSpec(item.path));
    })().catch((e) => reportTask(e));
  };

  const cutoutAll = (targets: GalleryItem[]) => {
    if (!targets.length) return;
    (async () => {
      for (const item of targets) await api.queueAdd(cutoutSpec(item.path));
    })().catch((e) => reportTask(e));
  };

  const deleteSelected = () => {
    if (selectedItems.length) setPendingDelete(selectedItems);
  };

  const copySelected = () => {
    if (!photos.length) return;
    run(api.galleryCopyFiles(photos.map((item) => item.path)));
  };

  const pinSelected = () => {
    if (!photos.length) return;
    run(api.galleryPinMany(photos.map((item) => item.path)).then(() => api.hide()));
  };

  return {
    items: viewable,
    visible,
    groups: groupBy(visible, period),
    empty: items.length === 0,
    noMatch: items.length > 0 && visible.length === 0,
    filter,
    setFilter: changeFilter,
    query,
    setQuery,
    resetFilters: () => {
      setQuery("");
      changeFilter("all");
    },
    tile,
    setTile: changeTile,
    period,
    setPeriod: changePeriod,
    collapsed,
    toggleGroup: (key: string) =>
      setCollapsed((prev) => {
        const next = new Set(prev);
        if (!next.delete(key)) next.add(key);
        return next;
      }),
    viewer,
    view: (item: GalleryItem) => setViewing(item.path),
    closeViewer: () => {
      setViewing(null);
      setTextOpen(false);
      setTranscriptOpen(false);
    },
    transcriptVideo,
    transcriptTrack,
    setTranscriptTrack,
    toggleTranscript: () => setTranscriptOpen((open) => !open),
    closeTranscript: () => setTranscriptOpen(false),
    text: textPath ? (texts.get(textPath) ?? {}) : null,
    toggleText: () => setTextOpen((open) => !open),
    closeText: () => setTextOpen(false),
    step,
    selected,
    selectMode,
    setSelectMode: changeSelectMode,
    selectedItems,
    selectedCount: selectedItems.length,
    photosSelected: photos.length > 0,
    videosSelected: videos.length > 0,
    gifTarget,
    closeGif: () => setGifTarget(null),
    trimTarget,
    closeTrim: () => setTrimTarget(null),
    compressTarget,
    closeCompress: () => setCompressTarget(null),
    translateTarget,
    closeTranslate: () => setTranslateTarget(null),
    subtitleTarget,
    closeSubtitle: () => setSubtitleTarget(null),
    addSelected,
    toggleSelect,
    selectRange,
    selectGroup,
    selectAll,
    clearSelection,
    pendingDelete,
    cancelDelete: () => setPendingDelete(null),
    confirmDelete,
    actions: {
      open: (item: GalleryItem) => run(api.galleryOpen(item.path)),
      reveal: (item: GalleryItem) => run(api.galleryReveal(item.path)),
      copy: (item: GalleryItem) => run(api.galleryCopy(item.path)),
      copySelected,
      extractText: (item: GalleryItem) => {
        setViewing(item.path);
        setTextOpen(true);
      },
      pin: (item: GalleryItem) => {
        api
          .galleryPin(item.path)
          .then(() => api.hide())
          .catch(report);
      },
      pinSelected,
      openGif: (item: GalleryItem) => setGifTarget(item),
      exportGif,
      openTrim: (item: GalleryItem) => setTrimTarget(item),
      trim,
      exportGifSelected,
      openCompress: (item: GalleryItem) => setCompressTarget(item),
      compress,
      compressFailed,
      openSubtitle: (item: GalleryItem, ms: number, text: string, track: Track) => {
        const tracks = subtitleTracks(item);
        setSubtitleTarget({ item, ms, text, track, tracks: tracks.includes(track) ? tracks : [...tracks, track] });
      },
      openSubtitleFromTranscript: (item: GalleryItem) => {
        const track: Track = item.transcript ? "original" : "english";
        api
          .transcriptRead(item.path, track)
          .then((segments) => {
            const cue = segments?.find((s) => s.text.trim());
            if (!cue) throw "NO_TRANSCRIPT";
            setSubtitleTarget({ item, ms: cue.start_ms, text: cue.text, track, tracks: subtitleTracks(item) });
          })
          .catch((e) => reportTask(e));
      },
      exportSubtitle,
      compressSelected,
      transcribe: (item: GalleryItem) => transcribeAll([item]),
      translate: (item: GalleryItem, target: string | null = null) => {
        setTranslateTarget(null);
        if (target) setTranscriptTrack(target);
        api.queueAdd(translateSpec(item.path, target)).catch((e) => reportTask(e));
      },
      openTranslate: (item: GalleryItem) => setTranslateTarget(item),
      transcribeSelected: () => transcribeAll(videos),
      removeBackground: (item: GalleryItem) => cutoutAll([item]),
      removeBackgroundSelected: () => cutoutAll(photos),
      deleteSelected,
      remove: (item: GalleryItem) => setPendingDelete([item]),
    },
  };
}

export type GalleryActions = ReturnType<typeof useGallery>["actions"];
