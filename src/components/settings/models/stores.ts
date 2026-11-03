import { useSettingsContext } from "@/components/settings-provider";
import type { PaddleSize, WhisperModel } from "@/lib/api";
import { PADDLE_SIZES, paddleLabels } from "./paddle-labels";
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
  progress: { id: string; done: number; total: number } | null;
  error: { id: string; message: string } | null;
  select: (id: string) => void;
  download: (id: string) => void;
  cancel: () => void;
  remove: (id: string) => void;
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
    progress: whisper.progress && { id: whisper.progress.model, done: whisper.progress.done, total: whisper.progress.total },
    error: whisper.error && { id: whisper.error.model, message: whisper.error.message },
    select: (id) => update({ ...cfg, whisper_model: id as WhisperModel }),
    download: (id) => whisper.download(id as WhisperModel),
    cancel: whisper.cancel,
    remove: (id) => whisper.remove(id as WhisperModel),
  };
}
