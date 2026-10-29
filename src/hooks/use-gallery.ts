import { useCallback, useEffect, useState } from "react";
import { api, type BulkFailure, type GalleryItem } from "@/lib/api";
import { fileName, groupBy, isViewable, matches, type Filter, type Period } from "@/lib/gallery";

export function useGallery() {
  const [items, setItems] = useState<GalleryItem[]>([]);
  const [filter, setFilter] = useState<Filter>("all");
  const [period, setPeriod] = useState<Period>("day");
  const [collapsed, setCollapsed] = useState<Set<string>>(new Set());
  const [viewing, setViewing] = useState<string | null>(null);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [selectMode, setSelectMode] = useState(false);
  const [anchor, setAnchor] = useState<string | null>(null);
  const [pendingDelete, setPendingDelete] = useState<GalleryItem[] | null>(null);
  const [error, setError] = useState("");

  const report = useCallback((e: unknown) => setError(String(e)), []);

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

  const visible = items.filter((i) => matches(i, filter));
  const viewable = visible.filter(isViewable);
  const index = viewable.findIndex((i) => i.path === viewing);
  const viewer = index < 0 ? null : { item: viewable[index], index, count: viewable.length };
  const selectedItems = visible.filter((i) => selected.has(i.path));
  const photos = selectedItems.filter((i) => i.kind === "photo");

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
    setError("");
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
    setError("");
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
      setError(failures.map((failure) => `${fileName(failure.path)}: ${failure.error}`).join(" · "));
    }
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
    empty: visible.length === 0,
    filter,
    setFilter: changeFilter,
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
    closeViewer: () => setViewing(null),
    step,
    error,
    selected,
    selectMode,
    setSelectMode: changeSelectMode,
    selectedItems,
    selectedCount: selectedItems.length,
    photosSelected: photos.length > 0,
    addSelected,
    toggleSelect,
    selectRange,
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
      pin: (item: GalleryItem) => {
        setError("");
        api
          .galleryPin(item.path)
          .then(() => api.hide())
          .catch(report);
      },
      pinSelected,
      deleteSelected,
      remove: (item: GalleryItem) => setPendingDelete([item]),
    },
  };
}

export type GalleryActions = ReturnType<typeof useGallery>["actions"];
