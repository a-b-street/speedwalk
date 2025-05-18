import { type Writable, writable } from "svelte/store";
import type { Feature, LineString } from "geojson";
import { Speedwalk } from "backend";

export interface NodeProps {
  id: number;
  tags: Record<string, string>;
}

export interface WayProps {
  id: number;
  tags: Record<string, string>;
  kind:
    | "sidewalk"
    | "good_roadway"
    | "quickfix_roadway"
    | "bad_roadway"
    | "other";
  fix?: keyof typeof quickfixes;
  problem?: keyof typeof problems;
}

export let colors = {
  sidewalk: "black",
  good_roadway: "green",
  quickfix_roadway: "pink",
  bad_roadway: "red",
  other: "grey",
};

export let quickfixes = {
  OldSidewalkSeparate: "Replace sidewalk=separate with sidewalk:both=separate",
  OldSidewalkNo: "Replace sidewalk=no with sidewalk:both=no",
  OldSidewalkNone: "Replace sidewalk=none with sidewalk:both=none",
};

export let problems = {
  DoubleTaggedLeftBoth: "Double-tagged: sidewalk:left and sidewalk:both",
  DoubleTaggedRightBoth: "Double-tagged: sidewalk:right and sidewalk:both",
  OldStyleSidewalk: "Old-style sidewalk tag included",
  MissingNewStyle: "New-style tags missing on one or both sides",
};

export function sum(list: number[]): number {
  return list.reduce((total, x) => total + x, 0);
}

export function prettyPrintDistance(meters: number): string {
  if (meters < 1000.0) {
    return Math.round(meters) + "m";
  }
  return (meters / 1000.0).toFixed(1) + "km";
}

export let backend: Writable<Speedwalk | null> = writable(null);
export let previewSidewalk: Writable<Feature<LineString> | null> =
  writable(null);
