import { ItemGroup } from "@/components/ui/item";
import { FpsRow } from "./fps-row";

export function Recording() {
  return (
    <ItemGroup className="gap-3">
      <FpsRow />
    </ItemGroup>
  );
}
