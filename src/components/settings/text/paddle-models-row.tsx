import { useState } from "react";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Item, ItemActions, ItemContent, ItemGroup, ItemTitle } from "@/components/ui/item";
import { useSettingsContext } from "@/components/settings-provider";
import type { PaddleSize } from "@/lib/api";
import { paddleLabels } from "./paddle-labels";

const SIZES: PaddleSize[] = ["tiny", "small", "medium"];

const mb = (bytes: number) => `${(bytes / 1048576).toFixed(bytes < 10485760 ? 1 : 0)} MB`;

export function PaddleModelsRow() {
  const { cfg, t, paddle } = useSettingsContext();
  const { sizes } = paddleLabels(t);
  const [target, setTarget] = useState<PaddleSize | null>(null);
  const downloading = paddle.progress?.size;

  return (
    <>
      <ItemGroup className="gap-1">
        <span className="px-1 text-sm font-medium">{t.paddleModels}</span>
        {SIZES.map((s) => {
          const ready = paddle.status.ready.includes(s);
          const inUse = ready && cfg.paddle_model === s;
          return (
            <Item key={s} variant="muted" size="sm">
              <ItemContent>
                <ItemTitle>
                  {sizes[s]}
                  <Badge variant={inUse ? "default" : "secondary"}>
                    {inUse ? t.paddleInUse : ready ? t.paddleDownloaded : t.paddleNotDownloaded}
                  </Badge>
                </ItemTitle>
              </ItemContent>
              <ItemActions className="gap-3">
                {ready && <span className="text-sm text-muted-foreground">{mb(paddle.status.bytes[s])}</span>}
                {ready && (
                  <Button size="sm" variant="outline" disabled={downloading === s} onClick={() => setTarget(s)}>
                    {t.paddleDelete}
                  </Button>
                )}
              </ItemActions>
            </Item>
          );
        })}
      </ItemGroup>
      <AlertDialog open={target !== null} onOpenChange={(open) => !open && setTarget(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{target && t.paddleDeleteTitle.replace("{size}", sizes[target])}</AlertDialogTitle>
            <AlertDialogDescription>
              {target && t.paddleDeleteHint.replace("{bytes}", mb(paddle.status.bytes[target]))}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t.cancel}</AlertDialogCancel>
            <AlertDialogAction
              variant="destructive"
              onClick={() => {
                if (target) paddle.remove(target);
                setTarget(null);
              }}
            >
              {t.paddleDelete}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
