import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { Video, VideoOff } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Item, ItemActions, ItemContent, ItemDescription, ItemFooter, ItemTitle } from "@/components/ui/item";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { CameraFeed } from "@/components/camera-feed";
import { useSettingsContext } from "@/components/settings-provider";
import type { CameraEffect, CameraLook, Config } from "@/lib/api";
import type { Strings } from "@/lib/i18n";
import { SliderField } from "../slider-field";

function usePreview(device: string, active: boolean) {
  const [stream, setStream] = useState<MediaStream | null>(null);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    if (!active) return;
    let alive = true;
    let current: MediaStream | null = null;
    setFailed(false);
    const media = navigator.mediaDevices;
    media
      .getUserMedia({ video: device ? { deviceId: { exact: device } } : true })
      .catch(() => (device ? media.getUserMedia({ video: true }) : Promise.reject()))
      .then((next) => {
        if (!alive) {
          next.getTracks().forEach((track) => track.stop());
          return;
        }
        current = next;
        setStream(next);
      })
      .catch(() => alive && setFailed(true));
    return () => {
      alive = false;
      current?.getTracks().forEach((track) => track.stop());
      setStream(null);
    };
  }, [device, active]);

  return { stream, failed };
}

const EFFECTS: CameraEffect[] = ["none", "blur", "image"];
const IMAGE_EXTS = ["png", "jpg", "jpeg", "webp", "bmp"];

type Knob = {
  look: keyof CameraLook;
  field: "record_blur_strength" | "record_blur_smoothing" | "record_bg_edge" | "record_bg_feather" | "record_bg_wrap" | "record_bg_wrap_radius";
  label: keyof Strings;
  min: number;
  max: number;
  unit: string;
  modes: CameraEffect[];
};

const KNOBS: Knob[] = [
  { look: "strength", field: "record_blur_strength", label: "cameraBlurStrength", min: 1, max: 10, unit: "", modes: ["blur"] },
  { look: "smoothing", field: "record_blur_smoothing", label: "cameraBlurSmoothing", min: 0, max: 90, unit: "%", modes: ["blur", "image"] },
  { look: "edge", field: "record_bg_edge", label: "cameraEdge", min: 10, max: 60, unit: "", modes: ["image"] },
  { look: "feather", field: "record_bg_feather", label: "cameraFeather", min: 0, max: 4, unit: "px", modes: ["image"] },
  { look: "wrap", field: "record_bg_wrap", label: "cameraWrap", min: 0, max: 100, unit: "%", modes: ["image"] },
  { look: "wrap_radius", field: "record_bg_wrap_radius", label: "cameraWrapRadius", min: 1, max: 10, unit: "", modes: ["image"] },
];

function lookOf(cfg: Config): CameraLook {
  return Object.fromEntries(KNOBS.map((k) => [k.look, cfg[k.field]])) as CameraLook;
}

export function CameraBlurRow() {
  const { cfg, t, update } = useSettingsContext();
  const effect: CameraEffect =
    cfg.record_camera_effect === "image" && !cfg.record_camera_image ? "none" : cfg.record_camera_effect;
  const on = effect !== "none";
  const labels: Record<CameraEffect, string> = {
    none: t.cameraEffectNone,
    blur: t.cameraEffectBlur,
    image: t.cameraEffectImage,
  };
  const [previewing, setPreviewing] = useState(false);
  const saved = lookOf(cfg);
  const key = JSON.stringify(saved);
  const [draft, setDraft] = useState(saved);
  const { stream, failed } = usePreview(cfg.record_camera, on && previewing);

  useEffect(() => {
    setDraft(JSON.parse(key));
  }, [key]);

  useEffect(() => {
    const stop = () => setPreviewing(false);
    const hidden = () => document.hidden && stop();
    window.addEventListener("blur", stop);
    document.addEventListener("visibilitychange", hidden);
    return () => {
      window.removeEventListener("blur", stop);
      document.removeEventListener("visibilitychange", hidden);
    };
  }, []);

  useEffect(() => {
    if (!on) setPreviewing(false);
  }, [on]);

  const browse = async () => {
    const picked = await open({ filters: [{ name: "Image", extensions: IMAGE_EXTS }] });
    if (typeof picked === "string") update({ ...cfg, record_camera_effect: "image", record_camera_image: picked });
  };

  const change = (value: string[]) => {
    const next = value[0] as CameraEffect | undefined;
    if (!next || next === effect) return;
    if (next === "image" && !cfg.record_camera_image) browse();
    else update({ ...cfg, record_camera_effect: next });
  };

  const showing = previewing && !failed;

  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.cameraBlur}</ItemTitle>
        <ItemDescription>{t.cameraBlurHint}</ItemDescription>
      </ItemContent>
      <ItemActions>
        <ToggleGroup variant="outline" size="sm" value={[effect]} onValueChange={change}>
          {EFFECTS.map((e) => (
            <ToggleGroupItem
              key={e}
              value={e}
              className="text-muted-foreground aria-pressed:border-primary aria-pressed:bg-primary aria-pressed:text-primary-foreground"
            >
              {labels[e]}
            </ToggleGroupItem>
          ))}
        </ToggleGroup>
      </ItemActions>
      {on && (
        <ItemFooter className="items-center gap-4">
          <div className="flex size-28 shrink-0 items-center justify-center overflow-hidden rounded-full bg-muted text-muted-foreground">
            {showing ? (
              <CameraFeed stream={stream} effect={effect} options={draft} image={cfg.record_camera_image} />
            ) : failed ? (
              <span className="px-3 text-center text-xs">{t.cameraPreviewFailed}</span>
            ) : (
              <VideoOff className="size-6" />
            )}
          </div>
          <div className="flex min-w-0 flex-1 flex-col items-start gap-3">
            {effect === "image" && (
              <div className="flex w-full items-center gap-2">
                <span className="min-w-0 flex-1 truncate text-muted-foreground">{cfg.record_camera_image}</span>
                <Button size="sm" variant="outline" onClick={browse}>
                  {t.browse}
                </Button>
              </div>
            )}
            {KNOBS.filter((k) => k.modes.includes(effect)).map((k) => (
              <SliderField
                key={k.field}
                label={t[k.label] as string}
                value={cfg[k.field]}
                min={k.min}
                max={k.max}
                unit={k.unit}
                onDraft={(v) => setDraft((d) => ({ ...d, [k.look]: v }))}
                onCommit={(v) => update({ ...cfg, [k.field]: v })}
              />
            ))}
            <Button variant="outline" size="sm" onClick={() => setPreviewing((p) => !p)}>
              {showing ? <VideoOff /> : <Video />}
              {showing ? t.cameraPreviewStop : t.cameraPreview}
            </Button>
          </div>
        </ItemFooter>
      )}
    </Item>
  );
}
