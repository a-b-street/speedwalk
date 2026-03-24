/**
 * Classify manual overrides against the map camera (viewport), using Turf:
 * - crossings: line–rectangle intersection
 * - deletions: stored midpoint vs viewport (no full way geometry in storage)
 */
import bboxPolygon from "@turf/bbox-polygon";
import booleanIntersects from "@turf/boolean-intersects";
import { lineString, point } from "@turf/helpers";
import {
  type AddedCrossingSegment,
  type DeletedWaySegment,
  isValidDeletedSegment,
  isValidSegment,
} from "./overridesSchema";

export type ViewportBounds = {
  west: number;
  south: number;
  east: number;
  north: number;
};

export function crossingStableId(seg: AddedCrossingSegment): string {
  if (seg.id) return seg.id;
  return `${seg.start.lng},${seg.start.lat}-${seg.end.lng},${seg.end.lat}`;
}

export function deletionStableId(seg: DeletedWaySegment): string {
  return seg.id ?? `${seg.wayId}-${seg.node1}-${seg.node2}`;
}

/** One bbox polygon; both sets for list row styling. */
export function inViewIdSets(
  crossings: AddedCrossingSegment[],
  deletions: DeletedWaySegment[],
  bounds: ViewportBounds,
): { crossingIds: Set<string>; deletionIds: Set<string> } {
  const poly = bboxPolygon([
    bounds.west,
    bounds.south,
    bounds.east,
    bounds.north,
  ]);
  const crossingIds = new Set<string>();
  for (const seg of crossings) {
    if (!isValidSegment(seg)) continue;
    const line = lineString([
      [seg.start.lng, seg.start.lat],
      [seg.end.lng, seg.end.lat],
    ]);
    if (booleanIntersects(line, poly)) crossingIds.add(crossingStableId(seg));
  }
  const deletionIds = new Set<string>();
  for (const seg of deletions) {
    if (!isValidDeletedSegment(seg)) continue;
    const pt = point([seg.midLng, seg.midLat]);
    if (booleanIntersects(pt, poly)) deletionIds.add(deletionStableId(seg));
  }
  return { crossingIds, deletionIds };
}
