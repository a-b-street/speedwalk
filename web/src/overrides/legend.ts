/** Shared legend colors for manual overrides map features. */
export const overridesLegendColors = {
  "Base data": "black",
  "Manually added crossing": "blue",
  "Manually deleted way": "#d73027",
} as const;

/** Legend items for LegendList (label + color). */
export const overridesLegendItems = Object.entries(overridesLegendColors).map(
  ([label, color]) => ({ label, color, swatchClass: "rectangle" as const }),
);
