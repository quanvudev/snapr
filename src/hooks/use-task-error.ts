import { useCallback, useRef } from "react";
import { toast } from "@/components/ui/toast";
import { useSettingsContext } from "@/components/settings-provider";
import { needsModels, taskError } from "@/lib/queue";

export function useTaskError() {
  const { t, setSection } = useSettingsContext();
  const strings = useRef(t);
  strings.current = t;

  return useCallback(
    (error: unknown, title = strings.current.galleryFailed, prefix = "") =>
      toast.add({
        type: "error",
        title,
        description: `${prefix}${taskError(strings.current, error)}`,
        actionProps: needsModels(error)
          ? { children: strings.current.openModels, onClick: () => setSection("models") }
          : undefined,
      }),
    [setSection],
  );
}
