import { useEffect, useRef, useState, type CSSProperties } from "react";
import { Ban } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { useSettingsContext } from "@/components/settings-provider";
import { api, type GalleryItem, type SubtitlePosition, type SubtitleSize, type SubtitleStyle, type Track } from "@/lib/api";
import { type SubtitleTarget } from "@/hooks/use-gallery";
import { activeSegment } from "@/hooks/use-transcript";
import { fileName } from "@/lib/gallery";
import { trackName } from "@/lib/languages";
import {
  BOX_PADDING_X_RATIO,
  BOX_PADDING_Y_RATIO,
  BOX_RADIUS_RATIO,
  EDGE_MARGIN_RATIO,
  LINE_HEIGHT_RATIO,
  loadSubtitlePreset,
  MAX_WIDTH_RATIO,
  MIN_TEXT_PX,
  MS_PER_SECOND,
  OUTLINE_RATIO,
  OUTLINE_STEPS,
  outlineColor,
  PERCENT,
  SUBTITLE_BACKGROUNDS,
  SUBTITLE_OPACITIES,
  SUBTITLE_POSITIONS,
  SUBTITLE_SIZE_RATIO,
  SUBTITLE_SIZES,
  SUBTITLE_TEXT_COLORS,
  type SubtitlePreset,
} from "@/lib/queue";

type Props = {
  target: SubtitleTarget | null;
  onClose: () => void;
  onExport: (item: GalleryItem, preset: SubtitlePreset, track: Track) => void;
};

type Dimensions = { intrinsicH: number; renderedW: number; renderedH: number };

const SWATCH = "size-4 rounded-full border border-foreground/20";

function rgba(hex: string, opacity: number) {
  const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));
  return `rgba(${r}, ${g}, ${b}, ${opacity / PERCENT})`;
}

function outline(style: SubtitleStyle, text: number) {
  const width = Math.max(1, OUTLINE_RATIO * text);
  const color = outlineColor(style.text_color);
  return Array.from({ length: OUTLINE_STEPS }, (_, i) => {
    const angle = (i * 2 * Math.PI) / OUTLINE_STEPS;
    return `${(width * Math.cos(angle)).toFixed(2)}px ${(width * Math.sin(angle)).toFixed(2)}px 0 ${color}`;
  }).join(", ");
}

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
    background: preset.style.background ? rgba(preset.style.background, preset.style.opacity) : "transparent",
    textShadow: preset.style.background ? undefined : outline(preset.style, text),
    color: preset.style.text_color,
    fontWeight: "bold",
    fontSize: text,
    lineHeight: LINE_HEIGHT_RATIO,
    textAlign: "center",
    pointerEvents: "none",
  };
}

function SubtitleForm({ target, onClose, onExport }: { target: SubtitleTarget } & Omit<Props, "target">) {
  const { cfg, t } = useSettingsContext();
  const video = useRef<HTMLVideoElement>(null);
  const [intrinsicH, setIntrinsicH] = useState(0);
  const [rendered, setRendered] = useState<{ w: number; h: number } | null>(null);
  const [preset, setPreset] = useState(loadSubtitlePreset);
  const [track, setTrack] = useState<Track>(target.track);
  const [cue, setCue] = useState(target.text);

  const chooseTrack = (next: Track) => {
    setTrack(next);
    api
      .transcriptRead(target.item.path, next)
      .then((segments) => {
        const list = segments ?? [];
        const current = list[activeSegment(list, target.ms)];
        setCue((current?.text.trim() ? current : list.find((s) => s.text.trim()))?.text.trim() ?? "");
      })
      .catch(() => setCue(""));
  };

  useEffect(() => {
    const el = video.current;
    if (!el) return;
    const observer = new ResizeObserver(() => setRendered({ w: el.clientWidth, h: el.clientHeight }));
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  const dims = intrinsicH && rendered ? { intrinsicH, renderedW: rendered.w, renderedH: rendered.h } : null;
  const setStyle = (style: Partial<SubtitleStyle>) => setPreset({ ...preset, style: { ...preset.style, ...style } });

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
          {dims && <div style={cueStyle(dims, preset)}>{cue || t.subtitleSample}</div>}
        </div>
      </div>
      <div className="flex flex-wrap items-center gap-x-6 gap-y-3">
        {target.tracks.length > 1 && (
          <div className="flex items-center gap-2">
            <span>{t.subtitleTrack}</span>
            <Select value={track} onValueChange={(v) => v && chooseTrack(v as Track)}>
              <SelectTrigger>
                <SelectValue>{(v: Track) => trackName(v, cfg.language, t.trackOriginal, target.item.language)}</SelectValue>
              </SelectTrigger>
              <SelectContent>
                {target.tracks.map((tr) => (
                  <SelectItem key={tr} value={tr}>
                    {trackName(tr, cfg.language, t.trackOriginal, target.item.language)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        )}
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
      <div className="flex flex-wrap items-center gap-x-6 gap-y-3">
        <div className="flex items-center gap-2">
          <span>{t.subtitleText}</span>
          <ToggleGroup
            variant="outline"
            spacing={0}
            value={[preset.style.text_color]}
            onValueChange={(v) => v[0] && setStyle({ text_color: v[0] })}
          >
            {SUBTITLE_TEXT_COLORS.map((c) => (
              <ToggleGroupItem key={c.value} value={c.value} aria-label={t[c.label]} title={t[c.label]}>
                <span className={SWATCH} style={{ background: c.value }} />
              </ToggleGroupItem>
            ))}
          </ToggleGroup>
        </div>
        <div className="flex items-center gap-2">
          <span>{t.subtitleBackground}</span>
          <ToggleGroup
            variant="outline"
            spacing={0}
            value={[preset.style.background || "none"]}
            onValueChange={(v) => v[0] && setStyle({ background: v[0] === "none" ? "" : v[0] })}
          >
            {SUBTITLE_BACKGROUNDS.map((c) => (
              <ToggleGroupItem key={c.label} value={c.value || "none"} aria-label={t[c.label]} title={t[c.label]}>
                {c.value ? <span className={SWATCH} style={{ background: c.value }} /> : <Ban />}
              </ToggleGroupItem>
            ))}
          </ToggleGroup>
        </div>
        <div className="flex items-center gap-2">
          <span>{t.subtitleOpacity}</span>
          <ToggleGroup
            variant="outline"
            spacing={0}
            disabled={!preset.style.background}
            value={[String(preset.style.opacity)]}
            onValueChange={(v) => v[0] && setStyle({ opacity: Number(v[0]) })}
          >
            {SUBTITLE_OPACITIES.map((o) => (
              <ToggleGroupItem key={o} value={String(o)}>
                {o}%
              </ToggleGroupItem>
            ))}
          </ToggleGroup>
        </div>
      </div>
      <DialogFooter>
        <Button variant="outline" onClick={onClose}>
          {t.cancel}
        </Button>
        <Button onClick={() => onExport(target.item, preset, track)}>{t.exportSubtitles}</Button>
      </DialogFooter>
    </>
  );
}
