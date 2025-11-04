import type { Feature } from "geojson";

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
    | "RoadWithoutSidewalks"
    | "RoadUnknown"
    | "Sidewalk"
    | "Crossing"
    | "Other";
  modified: boolean;
  node_ids: number[];
  is_severance: boolean;
  problems: Problem[];
}

interface Problem {
  note: string;
  details: Feature[];
}

export let colors = {
  RoadWithSeparate: "purple",
  RoadWithTags: "blue",
  RoadWithoutSidewalks: "orange",
  RoadUnknown: "red",

  Sidewalk: "black",
  Crossing: "green",
  Other: "grey",
};

// TODO Simpler to hardcode from backend
export let problemTypes = [
  "missing crossing node",
  "missing footway=crossing",
  "possible separate sidewalk near way without it tagged",
];
