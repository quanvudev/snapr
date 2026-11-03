import { useEffect, useState } from "react";
import { Images, ListChecks, Search, SearchX, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty";
import { InputGroup, InputGroupAddon, InputGroupButton, InputGroupInput } from "@/components/ui/input-group";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Slider } from "@/components/ui/slider";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { useSettingsContext } from "@/components/settings-provider";
import { TILE, useGallery } from "@/hooks/use-gallery";
import { formatGroup, type Filter, type Period } from "@/lib/gallery";
import { cn } from "@/lib/utils";
import { BulkBar } from "./bulk-bar";
import { CompressDialog } from "./compress-dialog";
import { DeleteDialog } from "./delete-dialog";
import { GalleryGroup } from "./gallery-group";
import { GifDialog } from "./gif-dialog";
import { SubtitleDialog } from "./subtitle-dialog";
import { Viewer } from "./viewer";

export function Gallery() {
  const { t, cfg } = useSettingsContext();
  const g = useGallery();
  const selecting = g.selectMode || g.selectedCount > 0;
  const [origin, setOrigin] = useState<DOMRect | null>(null);

  useEffect(() => {
    if (g.viewer || !selecting) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.target instanceof HTMLInputElement) return;
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
  }, [g.clearSelection, g.selectAll, selecting, g.viewer]);

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
    <div className="flex min-h-0 flex-1 flex-col">
      {g.viewer && (
        <Viewer
          {...g.viewer}
          items={g.items}
          actions={g.actions}
          paused={!!g.pendingDelete || !!g.gifTarget || !!g.compressTarget || !!g.subtitleTarget}
          text={g.text}
          onToggleText={g.toggleText}
          onCloseText={g.closeText}
          transcriptOpen={!!g.transcriptVideo}
          onToggleTranscript={g.toggleTranscript}
          onCloseTranscript={g.closeTranscript}
          onStep={g.step}
          onSelect={g.view}
          origin={origin}
          onClose={() => {
            setOrigin(null);
            g.closeViewer();
          }}
        />
      )}
      <div className={cn("flex min-h-0 flex-1 flex-col animate-in duration-200 fade-in-0", g.viewer && "hidden")}>
        <div className="flex flex-wrap items-center gap-2 border-b px-6 py-3">
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
          <InputGroup className="w-56">
            <InputGroupAddon>
              <Search />
            </InputGroupAddon>
            <InputGroupInput
              value={g.query}
              placeholder={t.search}
              onChange={(e) => g.setQuery(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Escape" && g.query) {
                  e.preventDefault();
                  e.stopPropagation();
                  g.setQuery("");
                }
              }}
            />
            {g.query && (
              <InputGroupAddon align="inline-end">
                <InputGroupButton size="icon-xs" title={t.close} onClick={() => g.setQuery("")}>
                  <X />
                </InputGroupButton>
              </InputGroupAddon>
            )}
          </InputGroup>
          <div className="ml-auto flex items-center gap-3">
            <div className="w-28" title={t.tileSize}>
              <Slider
                aria-label={t.tileSize}
                min={TILE.min}
                max={TILE.max}
                value={[g.tile]}
                onValueChange={(v) => g.setTile(Array.isArray(v) ? v[0] : v)}
              />
            </div>
            {g.selectMode ? (
              <Button variant="secondary" size="sm" onClick={g.clearSelection}>
                <X />
                {t.bulkCancel}
              </Button>
            ) : (
              <Button variant="outline" size="sm" onClick={() => g.setSelectMode(true)}>
                <ListChecks />
                {t.bulkSelect}
              </Button>
            )}
          </div>
        </div>
        <div className="relative flex min-h-0 flex-1 flex-col">
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
          ) : g.noMatch ? (
            <Empty>
              <EmptyHeader>
                <EmptyMedia variant="icon">
                  <SearchX />
                </EmptyMedia>
                <EmptyTitle>{t.noMatch}</EmptyTitle>
                <EmptyDescription>{t.noMatchHint}</EmptyDescription>
              </EmptyHeader>
              <EmptyContent>
                <Button variant="outline" size="sm" onClick={g.resetFilters}>
                  {t.showAll}
                </Button>
              </EmptyContent>
            </Empty>
          ) : (
            <ScrollArea className="min-h-0 flex-1">
              <div className={`px-6 ${selecting ? "pb-24" : "pb-6"}`}>
                {g.groups.map((group) => (
                  <GalleryGroup
                    key={group.key}
                    group={group}
                    label={formatGroup(group.date_ms, g.period, cfg.language)}
                    tile={g.tile}
                    collapsed={g.collapsed.has(group.key)}
                    actions={g.actions}
                    selectMode={g.selectMode}
                    selected={g.selected}
                    onToggle={() => g.toggleGroup(group.key)}
                    onOpen={(item, from) => {
                      setOrigin(from);
                      g.view(item);
                    }}
                    onSelect={g.toggleSelect}
                    onSelectGroup={(on) => g.selectGroup(group.items, on)}
                    onRange={g.selectRange}
                    onLongPress={g.addSelected}
                  />
                ))}
              </div>
            </ScrollArea>
          )}
          {selecting && (
            <BulkBar
              count={g.selectedCount}
              photos={g.photosSelected}
              videos={g.videosSelected}
              onSelectAll={g.selectAll}
              onDelete={g.actions.deleteSelected}
              onCopy={g.actions.copySelected}
              onPin={g.actions.pinSelected}
              onExportGif={g.actions.exportGifSelected}
              onCompress={g.actions.compressSelected}
              onTranscribe={g.actions.transcribeSelected}
              onCancel={g.clearSelection}
            />
          )}
        </div>
      </div>
      <GifDialog item={g.gifTarget} onClose={g.closeGif} onExport={g.actions.exportGif} />
      <CompressDialog
        item={g.compressTarget}
        onClose={g.closeCompress}
onCompress={g.actions.compress}
        onError={g.actions.compressFailed}
      />
      <SubtitleDialog target={g.subtitleTarget} onClose={g.closeSubtitle} onExport={g.actions.exportSubtitle} />
      <DeleteDialog items={g.pendingDelete} onCancel={g.cancelDelete} onConfirm={g.confirmDelete} />
    </div>
  );
}
