import { useSettingsContext } from "@/components/settings-provider";
import type { CaptionLanguage, GpuRuntime, PaddleSize, WhisperModel } from "@/lib/api";
import { CAPTION_LANGUAGES, CAPTIONS_NAME } from "@/lib/captions";
import { languageName } from "@/lib/languages";
import { RUNTIME_NAMES } from "./acceleration-card";
import { PADDLE_SIZES, paddleLabels } from "./paddle-labels";
import { CUTOUT_ID, CUTOUT_NAME } from "@/lib/cutout";
import { HYMT_ID, HYMT_NAME } from "@/lib/hymt";
import { UPSCALE_ID, UPSCALE_NAME } from "@/lib/upscale";
import { mb, WHISPER_MODELS } from "./whisper-models";

export const PADDLE_NAME = "PaddleOCR";
export const WHISPER_NAME = "Whisper";

export type ModelOption = { id: string; label: string; detail?: string };

export type ModelStore = {
  name: string;
  options: ModelOption[];
  selected: string;
  ready: string[];
  bytes: Record<string, number>;
  progress: { id: string; done: number; total: number; label?: string } | null;
  error: { id: string; message: string } | null;
  select: (id: string) => void;
  download: (id: string) => void;
  cancel: () => void;
  remove: (id: string) => void;
  locked?: { ids: string[]; hint: string };
};

export function labelOf(store: ModelStore, id: string) {
  return store.options.find((option) => option.id === id)?.label ?? id;
}

export function usePaddleStore(): ModelStore {
  const { cfg, t, paddle, update } = useSettingsContext();
  const { sizes } = paddleLabels(t);
  return {
    name: PADDLE_NAME,
    options: PADDLE_SIZES.map((id) => ({ id, label: sizes[id] })),
    selected: cfg.paddle_model,
    ready: paddle.status.ready,
    bytes: paddle.status.bytes,
    progress: paddle.progress && { id: paddle.progress.size, done: paddle.progress.done, total: paddle.progress.total },
    error: paddle.error && { id: paddle.error.size, message: paddle.error.message },
    select: (id) => update({ ...cfg, paddle_model: id as PaddleSize }),
    download: (id) => paddle.download(id as PaddleSize),
    cancel: paddle.cancel,
    remove: (id) => paddle.remove(id as PaddleSize),
  };
}

export function useWhisperStore(): ModelStore {
  const { cfg, t, whisper, update } = useSettingsContext();
  return {
    name: WHISPER_NAME,
    options: WHISPER_MODELS.map(({ value, label }) => ({
      id: value,
      label: t[label],
      detail: whisper.status.bytes[value] > 0 ? mb(whisper.status.bytes[value]) : undefined,
    })),
    selected: cfg.whisper_model,
    ready: whisper.status.ready,
    bytes: whisper.status.bytes,
    progress: whisper.progress && {
      id: whisper.progress.model,
      done: whisper.progress.done,
      total: whisper.progress.total,
      label: whisper.progress.gpu ? `${t[labelKey(whisper.progress.model)]} (GPU)` : undefined,
    },
    error: whisper.error && { id: whisper.error.model, message: whisper.error.message },
    select: (id) => update({ ...cfg, whisper_model: id as WhisperModel }),
    download: (id) => whisper.download(id as WhisperModel),
    cancel: whisper.cancel,
    remove: (id) => whisper.remove(id as WhisperModel),
  };
}

const labelKey = (model: WhisperModel) => WHISPER_MODELS.find((m) => m.value === model)?.label ?? "whisperSmall";

export function useWhisperGpuStore(): ModelStore {
  const { cfg, t, whisper, acceleration } = useSettingsContext();
  const onCuda = (acceleration.status.loaded ?? acceleration.status.resolved) === "cuda";
  return {
    name: t.whisperGpu,
    options: WHISPER_MODELS.map(({ value, label }) => ({ id: value, label: t[label] })),
    selected: onCuda ? cfg.whisper_model : "",
    ready: whisper.status.gpu_ready,
    bytes: whisper.status.gpu_bytes,
    progress: null,
    error: null,
    select: () => {},
    download: (id) => whisper.download(id as WhisperModel, true),
    cancel: whisper.cancel,
    remove: (id) => whisper.remove(id as WhisperModel, true),
  };
}

export function useRuntimeStore(): ModelStore {
  const { t, acceleration } = useSettingsContext();
  const { status, progress } = acceleration;
  const loaded = status.loaded ?? "";
  return {
    name: "",
    options: status.runtimes.map((r) => ({ id: r.id, label: t.accelRuntime.replace("{name}", RUNTIME_NAMES[r.id]) })),
    selected: loaded,
    ready: status.runtimes.filter((r) => r.ready).map((r) => r.id),
    bytes: Object.fromEntries(status.runtimes.map((r) => [r.id, r.disk])),
    progress: progress && { id: progress.runtime, done: progress.done, total: progress.total },
    error: null,
    select: () => {},
    download: (id) => acceleration.download(id as GpuRuntime),
    cancel: acceleration.cancel,
    remove: (id) => acceleration.remove(id as GpuRuntime),
    locked: { ids: loaded && loaded !== "cpu" ? [loaded] : [], hint: t.accelLoadedHint },
  };
}

export function useTranslateStore(askLicense: () => void): ModelStore {
  const { cfg, translator } = useSettingsContext();
  const { status, progress, error } = translator;
  return {
    name: HYMT_NAME,
    options: [{ id: HYMT_ID, label: HYMT_NAME, detail: status.bytes > 0 ? mb(status.bytes) : undefined }],
    selected: HYMT_ID,
    ready: status.ready ? [HYMT_ID] : [],
    bytes: { [HYMT_ID]: status.bytes },
    progress: progress && { id: HYMT_ID, done: progress.done, total: progress.total },
    error: error ? { id: HYMT_ID, message: error } : null,
    select: () => {},
    download: () => (cfg.hymt_license_accepted ? translator.download() : askLicense()),
    cancel: translator.cancel,
    remove: () => translator.remove(),
  };
}

export function useCutoutStore(): ModelStore {
  const { cutout } = useSettingsContext();
  const { status, progress, error } = cutout;
  return {
    name: CUTOUT_NAME,
    options: [{ id: CUTOUT_ID, label: CUTOUT_NAME, detail: status.bytes > 0 ? mb(status.bytes) : undefined }],
    selected: CUTOUT_ID,
    ready: status.ready ? [CUTOUT_ID] : [],
    bytes: { [CUTOUT_ID]: status.bytes },
    progress: progress && { id: CUTOUT_ID, done: progress.done, total: progress.total },
    error: error ? { id: CUTOUT_ID, message: error } : null,
    select: () => {},
    download: () => cutout.download(),
    cancel: cutout.cancel,
    remove: () => cutout.remove(),
  };
}

export function useCaptionsStore(): ModelStore {
  const { cfg, captions, update } = useSettingsContext();
  const { status, progress, error } = captions;
  const bytes = Object.fromEntries(status.models.map((m) => [m.language, m.bytes]));
  return {
    name: CAPTIONS_NAME,
    options: CAPTION_LANGUAGES.map((id) => ({
      id,
      label: languageName(id, cfg.language),
      detail: bytes[id] > 0 ? mb(bytes[id]) : undefined,
    })),
    selected: CAPTION_LANGUAGES.includes(cfg.captions_language as CaptionLanguage) ? cfg.captions_language : "en",
    ready: status.models.filter((m) => m.ready).map((m) => m.language),
    bytes,
    progress: progress && { id: progress.language, done: progress.done, total: progress.total },
    error: error && { id: error.language, message: error.message },
    select: (id) => update({ ...cfg, captions_language: id }),
    download: (id) => captions.download(id as CaptionLanguage),
    cancel: captions.cancel,
    remove: (id) => captions.remove(id as CaptionLanguage),
  };
}

export function useUpscaleStore(): ModelStore {
  const { upscale } = useSettingsContext();
  const { status, progress, error } = upscale;
  return {
    name: UPSCALE_NAME,
    options: [{ id: UPSCALE_ID, label: UPSCALE_NAME, detail: status.bytes > 0 ? mb(status.bytes) : undefined }],
    selected: UPSCALE_ID,
    ready: status.ready ? [UPSCALE_ID] : [],
    bytes: { [UPSCALE_ID]: status.bytes },
    progress: progress && { id: UPSCALE_ID, done: progress.done, total: progress.total },
    error: error ? { id: UPSCALE_ID, message: error } : null,
    select: () => {},
    download: () => upscale.download(),
    cancel: upscale.cancel,
    remove: () => upscale.remove(),
  };
}
