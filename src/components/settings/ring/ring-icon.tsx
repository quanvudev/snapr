import { useEffect, useState } from "react";
import { api, type RingItem } from "@/lib/api";
import { BUILTINS, GENERIC_APP_ICON } from "@/lib/ring";

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
  return src ? <img src={src} alt="" className="size-5" /> : <GENERIC_APP_ICON />;
}

export function RingIcon({ item }: { item: RingItem }) {
  if (item.kind === "app") return <AppIcon path={item.path} />;
  const Icon = BUILTINS[item.id].icon;
  return <Icon />;
}
