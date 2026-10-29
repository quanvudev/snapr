import { useEffect, useState } from "react";
import { api, type RingItem } from "@/lib/api";
import { BUILTINS, GENERIC_APP_GLYPH } from "@/lib/ring";

const ICON_FONT = '"Segoe Fluent Icons", "Segoe MDL2 Assets"';

export function Glyph({ char }: { char: string }) {
  return (
    <span aria-hidden="true" className="text-base leading-none" style={{ fontFamily: ICON_FONT }}>
      {char}
    </span>
  );
}

export function GenericAppIcon() {
  return <Glyph char={GENERIC_APP_GLYPH} />;
}

export function AppIcon({ path }: { path: string }) {
  const [src, setSrc] = useState<string | null>(null);
  useEffect(() => {
    let url: string | null = null;
    let live = true;
    api.ringIcon(path).then(
      (bytes) => {
        if (!live) return;
        url = URL.createObjectURL(new Blob([bytes], { type: "image/png" }));
        setSrc(url);
      },
      () => live && setSrc(null),
    );
    return () => {
      live = false;
      if (url) URL.revokeObjectURL(url);
    };
  }, [path]);
  return src ? <img src={src} alt="" className="size-5" /> : <GenericAppIcon />;
}

export function RingIcon({ item }: { item: RingItem }) {
  if (item.kind === "app") return <AppIcon path={item.path} />;
  return <Glyph char={BUILTINS[item.id].glyph} />;
}
