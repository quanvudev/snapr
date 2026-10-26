import { useCallback, useEffect, useState } from "react";
import { api, type GalleryItem } from "@/lib/api";
import { groupBy, isViewable, matches, type Filter, type Period } from "@/lib/gallery";

export function useGallery() {
  const [items, setItems] = useState<GalleryItem[]>([]);
  const [filter, setFilter] = useState<Filter>("all");
  const [period, setPeriod] = useState<Period>("day");
  const [collapsed, setCollapsed] = useState<Set<string>>(new Set());
  const [viewing, setViewing] = useState<string | null>(null);
  const [pendingDelete, setPendingDelete] = useState<GalleryItem | null>(null);
  const [error, setError] = useState("");

  const report = useCallback((e: unknown) => setError(String(e)), []);

  const reload = useCallback(() => {
    api.galleryList().then(setItems).catch(report);
  }, [report]);

  useEffect(() => {
    reload();
    const subs = [api.onSettingsOpen(reload), api.onGalleryChanged(reload)];
    return () => {
      subs.forEach((s) => s.then((f) => f()));
    };
  }, [reload]);

  const visible = items.filter((i) => matches(i, filter));
  const viewable = visible.filter(isViewable);
  const index = viewable.findIndex((i) => i.path === viewing);
  const viewer = index < 0 ? null : { item: viewable[index], index, count: viewable.length };

  const run = (p: Promise<unknown>) => {
    setError("");
    p.catch(report);
  };

  const step = (delta: number) => {
    const target = viewable[index + delta];
    if (target) setViewing(target.path);
  };

  const confirmDelete = async () => {
    const item = pendingDelete;
    if (!item) return;
    setPendingDelete(null);
    setError("");
    try {
      await api.galleryDelete(item.path);
    } catch (e) {
      report(e);
      return;
    }
    if (item.path === viewing) setViewing((viewable[index + 1] ?? viewable[index - 1])?.path ?? null);
    setItems((prev) => prev.filter((i) => i.path !== item.path));
  };

  return {
    items: viewable,
    groups: groupBy(visible, period),
    empty: visible.length === 0,
    filter,
    setFilter,
    period,
    setPeriod,
    collapsed,
    toggleGroup: (key: string) =>
      setCollapsed((prev) => {
        const next = new Set(prev);
        if (!next.delete(key)) next.add(key);
        return next;
      }),
    viewer,
    view: (item: GalleryItem) => setViewing(item.path),
    closeViewer: () => setViewing(null),
    step,
    error,
    pendingDelete,
    cancelDelete: () => setPendingDelete(null),
    confirmDelete,
    actions: {
      open: (item: GalleryItem) => run(api.galleryOpen(item.path)),
      reveal: (item: GalleryItem) => run(api.galleryReveal(item.path)),
      copy: (item: GalleryItem) => run(api.galleryCopy(item.path)),
      remove: setPendingDelete,
    },
  };
}

export type GalleryActions = ReturnType<typeof useGallery>["actions"];
