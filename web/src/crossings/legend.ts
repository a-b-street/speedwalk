import type { ExpressionSpecification } from "maplibre-gl";

/** Shared legend label → color for crossing-related map features and legend UI. */
export const crossingLegendColors = {
  "Junction to audit": "black",
  "Fully mapped junction": "green",
  Crossing: "yellow",
  "crossing=no": "purple",
  "crossing=generated": "cyan",
} as const;

/** Legend items for LegendList (label + color). */
export const crossingLegendItems = Object.entries(crossingLegendColors).map(
  ([label, color]) => ({ label, color }),
);

const crossingNodeColorExpression: ExpressionSpecification = [
  "case",
  ["get", "is_explicit_crossing_no"],
  crossingLegendColors["crossing=no"],
  ["get", "is_generated_crossing"],
  crossingLegendColors["crossing=generated"],
  crossingLegendColors["Crossing"],
];

/** Circle radius for crossing nodes on the map (small nodes at junctions). */
export const crossingNodeCircleRadius = 7;

/**
 * Returns paint props for a CircleLayer of crossing nodes (yellow / purple / cyan by type).
 * Pass an opacity expression (e.g. 1 or hoverStateFilter(0.3, 1.0)).
 */
export function getCrossingNodeCirclePaint(
  circleOpacity: ExpressionSpecification,
): Record<string, unknown> {
  return {
    "circle-radius": crossingNodeCircleRadius,
    "circle-color": crossingNodeColorExpression,
    "circle-opacity": circleOpacity,
    "circle-stroke-color": "black",
    "circle-stroke-width": 1,
  };
}
