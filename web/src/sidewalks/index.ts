import type { Feature } from "geojson";
import { constructMatchExpression } from "svelte-utils/map";
import type { ExpressionSpecification } from "maplibre-gl";

export interface NodeProps {
  id: number;
  tags?: Record<string, string>;
  is_crossing: boolean;
  modified: boolean;
  way_ids: number[];
  problems: Problem[];
}

export interface WayProps {
  id: number;
  tags: Record<string, string>;
  kind:
    | "RoadWithSeparate"
    | "RoadWithTags"
    | "RoadWithoutSidewalksExplicit"
    | "RoadWithoutSidewalksImplicit"
    | "RoadUnknown"
    | "Sidewalk"
    | "Crossing"
    | "Other";
  modified: boolean;
  node_ids: number[];
  is_severance: boolean;
  is_service: boolean;
  problems: Problem[];
}

interface Problem {
  note: string;
  details: Feature[];
}

export let colors = {
  RoadWithSeparate: "purple",
  RoadWithTags: "blue",
  RoadWithoutSidewalksExplicit: "#8B4000",
  RoadWithoutSidewalksImplicit: "orange",
  RoadUnknown: "red",

  Sidewalk: "black",
  Crossing: "green",
  Other: "grey",
};

export let kindLabels = {
  RoadWithSeparate: "With separate sidewalks",
  RoadWithTags: "With tagged sidewalks",
  RoadWithoutSidewalksExplicit: "Tagged as no sidewalks",
  RoadWithoutSidewalksImplicit: "Assumed as no sidewalks",
  RoadUnknown: "Totally unknown",
  Sidewalk: "Separate sidewalk",
  Crossing: "Crossing",
  Other: "Other",
};

// Zoom-dependant line width, adapted from from the Minor road layer (secondary
// road class) from https://api.maptiler.com/maps/streets-v2/style.json. At
// high zoom, make sidewalks, crossings, and other roads thinner.
export function roadLineWidth(extraWidth: number): ExpressionSpecification {
  return [
    "interpolate",
    ["linear"],
    ["zoom"],
    5,
    0.5 + extraWidth,
    10,
    1 + extraWidth,
    12,
    1.5 + extraWidth,
    14,
    4 + extraWidth,
    16,
    constructMatchExpression(
      ["get", "kind"],
      {
        Sidewalk: 4 + extraWidth,
        Crossing: 5 + extraWidth,
        Other: 5 + extraWidth,
      },
      7 + extraWidth,
    ),
    20,
    constructMatchExpression(
      ["get", "kind"],
      {
        Sidewalk: 7 + extraWidth,
        Crossing: 10 + extraWidth,
        Other: 10 + extraWidth,
      },
      24 + extraWidth,
    ),
  ] as ExpressionSpecification;
}
