import { useQueryState, parseAsStringLiteral } from "nuqs-svelte";

export const modes = [
  "sidewalks",
  "crossings",
  "disconnections",
  "export",
] as const;

export type ModeValue = typeof modes[number];

const modeParser = parseAsStringLiteral(modes).withDefault("sidewalks");

export function useModeState() {
  return useQueryState("mode", modeParser.withOptions({
    history: "push",
  }));
}
