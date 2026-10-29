import type { PaddleSize } from "@/lib/api";
import type { T } from "@/lib/i18n";

type Strings = (typeof T)["en"];

export function paddleLabels(t: Strings): { sizes: Record<PaddleSize, string>; descs: Record<PaddleSize, string> } {
  return {
    sizes: { tiny: t.sizeTiny, small: t.sizeSmall, medium: t.sizeMedium },
    descs: { tiny: t.descTiny, small: t.descSmall, medium: t.descMedium },
  };
}
