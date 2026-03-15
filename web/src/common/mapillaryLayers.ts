/**
 * Stable layer IDs for Mapillary pin layers (image points). Used so other modes
 * (e.g. Overrides) can detect clicks on Mapillary pins and avoid conflicting behavior.
 */
export const MAPILLARY_PIN_LAYER_IDS = {
  symbol: "mapillary-image-symbol",
  circleInteractive: "mapillary-image-circle-interactive",
  circle: "mapillary-image-circle",
} as const;

/** All pin layer IDs as array, for queryRenderedFeatures(layers: [...]). */
export const MAPILLARY_PIN_LAYER_IDS_LIST = Object.values(
  MAPILLARY_PIN_LAYER_IDS,
);
