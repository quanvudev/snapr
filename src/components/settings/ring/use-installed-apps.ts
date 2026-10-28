import { useEffect, useState } from "react";
import { api, type InstalledApp } from "@/lib/api";

let cache: InstalledApp[] | null = null;

export function useInstalledApps(open: boolean) {
  const [apps, setApps] = useState<InstalledApp[] | null>(cache);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    if (!open || cache) return;
    setFailed(false);
    api.listApps().then(
      (list) => {
        cache = list;
        setApps(list);
      },
      () => setFailed(true),
    );
  }, [open]);

  return { apps, failed, loading: !apps && !failed };
}
