import { useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Slider } from "@/components/ui/slider";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { useSettingsContext } from "@/components/settings-provider";
import { api, type GalleryItem } from "@/lib/api";
import { fileName, formatSize } from "@/lib/gallery";
import {
  GIF_FPS,
  GIF_SLIDER_STEP_MS,
  GIF_WIDTHS,
  gifEstimate,
  loadGifPreset,
  MS_PER_SECOND,
  type GifRange,
} from "@/lib/queue";

type Props = { item: GalleryItem | null; onClose: () => void; onExport: (item: GalleryItem, range: GifRange) => void };

const seconds = (ms: number) => `${(ms / MS_PER_SECOND).toFixed(1)}s`;
const widthKey = (width: number | null) => String(width);
const parseWidth = (key: string) => (key === "null" ? null : Number(key));

export function GifDialog({ item, onClose, onExport }: Props) {
  const { t } = useSettingsContext();
  return (
    <Dialog open={!!item} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-xl">
        <DialogHeader>
          <DialogTitle>{t.gifTitle}</DialogTitle>
        </DialogHeader>
        {item && <GifForm key={item.path} item={item} onClose={onClose} onExport={onExport} />}
      </DialogContent>
    </Dialog>
  );
}

function GifForm({ item, onClose, onExport }: { item: GalleryItem; onClose: () => void; onExport: Props["onExport"] }) {
  const { t } = useSettingsContext();
  const video = useRef<HTMLVideoElement>(null);
  const [meta, setMeta] = useState<{ w: number; h: number; durationMs: number } | null>(null);
  const [range, setRange] = useState<[number, number] | null>(null);
  const [preset, setPreset] = useState(loadGifPreset);

  const [start, end] = range ?? [0, meta?.durationMs ?? 0];
  const length = end - start;

  const changeRange = (value: number | readonly number[]) => {
    if (!Array.isArray(value)) return;
    const next: [number, number] = [value[0], value[1]];
    const moved = next[0] !== start ? next[0] : next[1];
    setRange(next);
    if (video.current) video.current.currentTime = moved / MS_PER_SECOND;
  };

  return (
    <>
      <video
        ref={video}
        src={api.mediaSrc(item.path)}
        title={fileName(item.path)}
        controls
        className="max-h-72 w-full rounded-lg bg-black object-contain"
        onLoadedMetadata={(e) =>
          setMeta({
            w: e.currentTarget.videoWidth,
            h: e.currentTarget.videoHeight,
            durationMs: Math.floor(e.currentTarget.duration * MS_PER_SECOND),
          })
        }
      />
      <div className="grid gap-2">
        <Slider
          aria-label={t.gifRange}
          disabled={!meta}
          min={0}
          max={meta?.durationMs ?? 1}
          step={GIF_SLIDER_STEP_MS}
          minStepsBetweenValues={1}
          value={[start, end]}
          onValueChange={changeRange}
        />
        <div className="flex justify-between text-xs text-muted-foreground tabular-nums">
          <span>
            {t.gifStart}: {seconds(start)}
          </span>
          <span>
            {t.gifDuration}: {seconds(length)}
          </span>
          <span>
            {t.gifEnd}: {seconds(end)}
          </span>
        </div>
      </div>
      <div className="flex flex-wrap items-center gap-x-6 gap-y-3">
        <div className="flex items-center gap-2">
          <span>{t.gifWidth}</span>
          <ToggleGroup
            variant="outline"
            spacing={0}
            value={[widthKey(preset.width)]}
            onValueChange={(v) => v[0] && setPreset({ ...preset, width: parseWidth(v[0]) })}
          >
            {GIF_WIDTHS.map((width) => (
              <ToggleGroupItem key={widthKey(width)} value={widthKey(width)}>
                {width ?? t.gifOriginal}
              </ToggleGroupItem>
            ))}
          </ToggleGroup>
        </div>
        <div className="flex items-center gap-2">
          <span>{t.gifFps}</span>
          <ToggleGroup
            variant="outline"
            spacing={0}
            value={[String(preset.fps)]}
            onValueChange={(v) => v[0] && setPreset({ ...preset, fps: Number(v[0]) })}
          >
            {GIF_FPS.map((fps) => (
              <ToggleGroupItem key={fps} value={String(fps)}>
                {fps}
              </ToggleGroupItem>
            ))}
          </ToggleGroup>
        </div>
      </div>
      {meta && (
        <p className="text-xs text-muted-foreground">
          {t.gifEstimate.replace("{size}", formatSize(gifEstimate(meta, length, preset)))}
        </p>
      )}
      <DialogFooter>
        <Button variant="outline" onClick={onClose}>
          {t.cancel}
        </Button>
        <Button disabled={!meta || length <= 0} onClick={() => onExport(item, { preset, startMs: start, endMs: end === meta?.durationMs ? null : end })}>
          {t.gifTitle}
        </Button>
      </DialogFooter>
    </>
  );
}
