import { useEffect } from "react";
import { Images } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { useSettingsContext } from "@/components/settings-provider";
import { useGallery } from "@/hooks/use-gallery";
import { formatGroup, type Filter, type Period } from "@/lib/gallery";
import { BulkBar } from "./bulk-bar";
import { DeleteDialog } from "./delete-dialog";
import { GalleryGroup } from "./gallery-group";
import { Viewer } from "./viewer";

export function Gallery() {
  const { t, cfg } = useSettingsContext();
  const g = useGallery();

  useEffect(() => {
    if (g.viewer || (!g.selectMode && g.selectedCount === 0)) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        event.stopPropagation();
        g.clearSelection();
      } else if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "a") {
        event.preventDefault();
        g.selectAll();
      }
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [g.clearSelection, g.selectAll, g.selectMode, g.selectedCount, g.viewer]);

  const filters: { id: Filter; label: string }[] = [
    { id: "all", label: t.filterAll },
    { id: "photo", label: t.filterPhotos },
    { id: "video", label: t.filterVideos },
  ];
  const periods: { id: Period; label: string }[] = [
    { id: "day", label: t.periodDay },
    { id: "month", label: t.periodMonth },
    { id: "year", label: t.periodYear },
  ];

  return (
    <div className="space-y-4">
      {g.viewer ? (
        <Viewer
          {...g.viewer}
          items={g.items}
          actions={g.actions}
          paused={!!g.pendingDelete}
          onStep={g.step}
          onSelect={g.view}
          onClose={g.closeViewer}
        />
      ) : (
        <>
          <div className="flex flex-wrap items-center justify-between gap-2">
            <div className="flex flex-wrap items-center gap-2">
              <ToggleGroup
                variant="outline"
                spacing={0}
                value={[g.filter]}
                onValueChange={(v) => v[0] && g.setFilter(v[0] as Filter)}
              >
                {filters.map((f) => (
                  <ToggleGroupItem key={f.id} value={f.id}>
                    {f.label}
                  </ToggleGroupItem>
                ))}
              </ToggleGroup>
              <ToggleGroup
                variant="outline"
                spacing={0}
                value={[g.period]}
                onValueChange={(v) => v[0] && g.setPeriod(v[0] as Period)}
              >
                {periods.map((p) => (
                  <ToggleGroupItem key={p.id} value={p.id}>
                    {p.label}
                  </ToggleGroupItem>
                ))}
              </ToggleGroup>
            </div>
            <Button
              variant={g.selectMode ? "secondary" : "outline"}
              size="sm"
              onClick={() => g.setSelectMode(!g.selectMode)}
            >
              {t.bulkSelect}
            </Button>
          </div>
          {(g.selectMode || g.selectedCount > 0) && (
            <BulkBar
              count={g.selectedCount}
              photos={g.photosSelected}
              onSelectAll={g.selectAll}
              onDelete={g.actions.deleteSelected}
              onCopy={g.actions.copySelected}
              onPin={g.actions.pinSelected}
              onCancel={g.clearSelection}
            />
          )}
          {g.empty ? (
            <Empty>
              <EmptyHeader>
                <EmptyMedia variant="icon">
                  <Images />
                </EmptyMedia>
                <EmptyTitle>{t.galleryEmpty}</EmptyTitle>
                <EmptyDescription>{t.galleryEmptyHint}</EmptyDescription>
              </EmptyHeader>
            </Empty>
          ) : (
            g.groups.map((group) => (
              <GalleryGroup
                key={group.key}
                group={group}
                label={formatGroup(group.date_ms, g.period, cfg.language)}
                collapsed={g.collapsed.has(group.key)}
                actions={g.actions}
                selectMode={g.selectMode}
                selected={g.selected}
                onToggle={() => g.toggleGroup(group.key)}
                onOpen={g.view}
                onSelect={g.toggleSelect}
                onRange={g.selectRange}
                onLongPress={g.addSelected}
              />
            ))
          )}
        </>
      )}
      {g.error && (
        <p className="text-xs text-red-500">
          {t.galleryFailed}: {g.error}
        </p>
      )}
      <DeleteDialog items={g.pendingDelete} onCancel={g.cancelDelete} onConfirm={g.confirmDelete} />
    </div>
  );
}
