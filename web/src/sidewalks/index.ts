import { type Writable, writable } from "svelte/store";
import type { FeatureCollection, LineString, Point } from "geojson";

export interface NodeProps {
  id: number;
  tags?: Record<string, string>;
  is_crossing: boolean;
  modified: boolean;
  way_ids: number[];
}

export interface WayProps {
  id: number;
  tags: Record<string, string>;
  kind:
    | "sidewalk"
    | "good_roadway"
    | "quickfix_roadway"
    | "old_style_roadway"
    | "bad_roadway"
    | "other";
  fix?: keyof typeof quickfixes;
  problem?: keyof typeof problems;
  modified: boolean;
}

export let colors = {
  sidewalk: "black",
  good_roadway: "green",
  quickfix_roadway: "pink",
  old_style_roadway: "purple",
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

export let previewSidewalk: Writable<FeatureCollection<
  LineString | Point
> | null> = writable(null);
