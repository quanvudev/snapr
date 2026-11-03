import { useEffect, useRef, useState, type CSSProperties } from "react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { useSettingsContext } from "@/components/settings-provider";
import { api, type GalleryItem, type SubtitlePosition, type SubtitleSize } from "@/lib/api";
import { type SubtitleTarget } from "@/hooks/use-gallery";
import { fileName } from "@/lib/gallery";
import {
  BOX_ALPHA,
  BOX_PADDING_X_RATIO,
  BOX_PADDING_Y_RATIO,
  BOX_RADIUS_RATIO,
  EDGE_MARGIN_RATIO,
  LINE_HEIGHT_RATIO,
  loadSubtitlePreset,
  MAX_WIDTH_RATIO,
  MIN_TEXT_PX,
  MS_PER_SECOND,
  SUBTITLE_POSITIONS,
  SUBTITLE_SIZE_RATIO,
  SUBTITLE_SIZES,
  type SubtitlePreset,
} from "@/lib/queue";

type Props = {
  target: SubtitleTarget | null;
  onClose: () => void;
  onExport: (item: GalleryItem, preset: SubtitlePreset) => void;
};

type Dimensions = { intrinsicH: number; renderedW: number; renderedH: number };

export function SubtitleDialog({ target, onClose, onExport }: Props) {
  const { t } = useSettingsContext();
  return (
    <Dialog open={!!target} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-xl">
        <DialogHeader>
          <DialogTitle>{t.exportSubtitles}</DialogTitle>
        </DialogHeader>
        {target && <SubtitleForm key={target.item.path} target={target} onClose={onClose} onExport={onExport} />}
      </DialogContent>
    </Dialog>
  );
}

function cueStyle(dims: Dimensions, preset: SubtitlePreset): CSSProperties {
  const scale = dims.renderedH / dims.intrinsicH;
  const text = Math.max(MIN_TEXT_PX, SUBTITLE_SIZE_RATIO[preset.size] * dims.intrinsicH) * scale;
  const margin = EDGE_MARGIN_RATIO * dims.renderedH;
  return {
    position: "absolute",
    left: "50%",
    transform: "translateX(-50%)",
    [preset.position]: margin,
    maxWidth: MAX_WIDTH_RATIO * dims.renderedW,
    width: "max-content",
    padding: `${BOX_PADDING_Y_RATIO * text}px ${BOX_PADDING_X_RATIO * text}px`,
    borderRadius: BOX_RADIUS_RATIO * text,
    background: `rgba(0, 0, 0, ${BOX_ALPHA})`,
    color: "white",
    fontWeight: "bold",
    fontSize: text,
    lineHeight: LINE_HEIGHT_RATIO,
    textAlign: "center",
    pointerEvents: "none",
  };
}

function SubtitleForm({ target, onClose, onExport }: { target: SubtitleTarget } & Omit<Props, "target">) {
  const { t } = useSettingsContext();
  const video = useRef<HTMLVideoElement>(null);
  const [intrinsicH, setIntrinsicH] = useState(0);
  const [rendered, setRendered] = useState<{ w: number; h: number } | null>(null);
  const [preset, setPreset] = useState(loadSubtitlePreset);

  useEffect(() => {
    const el = video.current;
    if (!el) return;
    const observer = new ResizeObserver(() => setRendered({ w: el.clientWidth, h: el.clientHeight }));
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  const dims = intrinsicH && rendered ? { intrinsicH, renderedW: rendered.w, renderedH: rendered.h } : null;

  return (
    <>
      <div className="flex justify-center">
        <div className="relative inline-flex max-w-full">
          <video
            ref={video}
            src={api.mediaSrc(target.item.path)}
            title={fileName(target.item.path)}
            preload="auto"
            muted
            className="max-h-72 max-w-full rounded-lg bg-black"
            onLoadedMetadata={(e) => {
              e.currentTarget.currentTime = target.ms / MS_PER_SECOND;
              setIntrinsicH(e.currentTarget.videoHeight);
            }}
          />
          {dims && <div style={cueStyle(dims, preset)}>{target.text || t.subtitleSample}</div>}
        </div>
      </div>
      <div className="flex flex-wrap items-center gap-x-6 gap-y-3">
        <div className="flex items-center gap-2">
          <span>{t.subtitlePosition}</span>
          <ToggleGroup
            variant="outline"
            spacing={0}
            value={[preset.position]}
            onValueChange={(v) => v[0] && setPreset({ ...preset, position: v[0] as SubtitlePosition })}
          >
            {SUBTITLE_POSITIONS.map((p) => (
              <ToggleGroupItem key={p.value} value={p.value}>
                {t[p.label]}
              </ToggleGroupItem>
            ))}
          </ToggleGroup>
        </div>
        <div className="flex items-center gap-2">
          <span>{t.subtitleSize}</span>
          <ToggleGroup
            variant="outline"
            spacing={0}
            value={[preset.size]}
            onValueChange={(v) => v[0] && setPreset({ ...preset, size: v[0] as SubtitleSize })}
          >
            {SUBTITLE_SIZES.map((s) => (
              <ToggleGroupItem key={s.value} value={s.value}>
                {t[s.label]}
              </ToggleGroupItem>
            ))}
          </ToggleGroup>
        </div>
      </div>
      <DialogFooter>
        <Button variant="outline" onClick={onClose}>
          {t.cancel}
        </Button>
        <Button onClick={() => onExport(target.item, preset)}>{t.exportSubtitles}</Button>
      </DialogFooter>
    </>
  );
}
