import type { WhisperModel } from "@/lib/api";
import type { Strings } from "@/lib/queue";

export const WHISPER_MODELS: { value: WhisperModel; label: keyof Strings }[] = [
  { value: "base", label: "whisperBase" },
  { value: "small", label: "whisperSmall" },
  { value: "turbo", label: "whisperTurbo" },
];

const BYTES_PER_MB = 1048576;
const SMALL_MB = 10;

export const mb = (bytes: number) => `${(bytes / BYTES_PER_MB).toFixed(bytes < SMALL_MB * BYTES_PER_MB ? 1 : 0)} MB`;
