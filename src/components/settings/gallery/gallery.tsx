import { Images } from "lucide-react";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { useSettingsContext } from "@/components/settings-provider";
import { useGallery } from "@/hooks/use-gallery";
import { formatGroup, type Filter, type Period } from "@/lib/gallery";
import { DeleteDialog } from "./delete-dialog";
import { GalleryGroup } from "./gallery-group";
import { Viewer } from "./viewer";

export function Gallery() {
  const { t, cfg } = useSettingsContext();
  const g = useGallery();
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
                onToggle={() => g.toggleGroup(group.key)}
                onOpen={g.view}
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
      <DeleteDialog item={g.pendingDelete} onCancel={g.cancelDelete} onConfirm={g.confirmDelete} />
    </div>
  );
}
