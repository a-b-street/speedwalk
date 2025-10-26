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
  kind: "Sidewalk" | "RoadWithSeparate" | "Road" | "Crossing" | "Other";
  modified: boolean;
  node_ids: number[];
}

export let colors = {
  Sidewalk: "black",
  RoadWithSeparate: "purple",
  Road: "red",
  Crossing: "green",
  Other: "grey",
};
