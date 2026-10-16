import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { FolderOpen } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Item, ItemContent, ItemDescription, ItemTitle } from "@/components/ui/item";
import { useSettingsContext } from "@/components/settings-provider";
import { api } from "@/lib/api";

export function FolderRow({ withOpen }: { withOpen?: boolean }) {
  const { cfg, t, dir, update } = useSettingsContext();
  const [draft, setDraft] = useState(cfg.output_dir);
  useEffect(() => setDraft(cfg.output_dir), [cfg.output_dir]);

  const commit = (output_dir: string) => {
    if (output_dir !== cfg.output_dir) update({ ...cfg, output_dir });
  };

  const browse = async () => {
    const picked = await open({ directory: true });
    if (typeof picked === "string") commit(picked);
  };

  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.folder}</ItemTitle>
        <div className="flex gap-2">
          <Input
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            onBlur={() => commit(draft)}
            onKeyDown={(e) => {
              if (e.key === "Enter") e.currentTarget.blur();
            }}
          />
          <Button variant="outline" onClick={browse}>
            {t.browse}
          </Button>
          {withOpen && (
            <Button variant="outline" onClick={() => api.openFolder(cfg.output_dir)}>
              <FolderOpen />
              {t.open}
            </Button>
          )}
        </div>
        <ItemDescription className="truncate">{dir}</ItemDescription>
      </ItemContent>
    </Item>
  );
}
