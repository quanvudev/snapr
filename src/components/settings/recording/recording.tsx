import { ItemGroup } from "@/components/ui/item";
import { CountdownRow } from "./countdown-row";
import { FpsRow } from "./fps-row";

export function Recording() {
  return (
    <ItemGroup className="gap-3">
      <FpsRow />
      <CountdownRow />
    </ItemGroup>
  );
}
