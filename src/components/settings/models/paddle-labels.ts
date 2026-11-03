import type { PaddleSize } from "@/lib/api";
import type { Strings } from "@/lib/queue";

export const PADDLE_SIZES: PaddleSize[] = ["tiny", "small", "medium"];

export function paddleLabels(t: Strings): { sizes: Record<PaddleSize, string>; descs: Record<PaddleSize, string> } {
  return {
    sizes: { tiny: t.sizeTiny, small: t.sizeSmall, medium: t.sizeMedium },
    descs: { tiny: t.descTiny, small: t.descSmall, medium: t.descMedium },
  };
}
