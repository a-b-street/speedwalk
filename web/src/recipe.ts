import type { CrossingScopeBulk } from "./";

// A Recipe is a portable, shareable sequence of map-editing operations. It is
// the mechanism by which one user can hand off a reproducible workflow to
// another: the steps a user applies interactively (generate crossings, make
// sidewalks, apply overrides, …) are recorded in the `recipeSteps` store
// (index.ts), then serialised into the URL by ExportMode. When a recipient
// opens that URL, App.svelte reads the `?recipe=` query param, decodes it into
// a `Recipe`, and stores it in `pendingRecipe`. RecipeCard then offers a
// one-click "apply" button that re-runs each step against the live backend.

// NetworkFilterInclude controls which edges are included in the routable
// network that gets exported. The value is carried by the `setNetworkFilter`
// step and forwarded to the backend graph-building logic.
export type NetworkFilterInclude =
  | "Everything"
  | "OnlyExplicitFootways"
  | "RouteableNetwork";

// RecipeStep is a discriminated union — each variant maps 1-to-1 to a backend
// mutation or a frontend filter operation. Adding a new operation means adding
// a branch here, a case in `stepLabel`, and a handler in RecipeCard's
// `executeStep`. The `op` field acts as the tag for exhaustive switching.
export type RecipeStep =
  | { op: "generateMissingCrossings"; scope: CrossingScopeBulk }
  | { op: "makeAllSidewalks"; onlyMajor: boolean }
  | { op: "connectAllCrossings"; includeCrossingNo: boolean }
  | { op: "assumeTags"; driveOnLeft: boolean }
  | { op: "applyOverrides" }
  | { op: "applyMaxspeed" }
  | { op: "applyCrossingNodeTags" }
  | {
      op: "setNetworkFilter";
      include: NetworkFilterInclude;
      ignore_deadends: boolean;
    };

// Recipe wraps the step list with a version field so that future breaking
// changes to the step schema can be detected and rejected gracefully in
// `decodeRecipe`. Currently only v:1 exists.
export interface Recipe {
  v: 1;
  steps: RecipeStep[];
}

// encodeRecipe serialises a Recipe to a URL-safe base64 string. The result is
// placed in the `?recipe=` query parameter by ExportMode when the user copies
// a recipe link.
export function encodeRecipe(recipe: Recipe): string {
  return btoa(JSON.stringify(recipe));
}

// decodeRecipe is the inverse of encodeRecipe. It is called in App.svelte on
// page load whenever a `?recipe=` param is present. Returns null for any
// malformed, tampered, or version-mismatched input so the app degrades
// gracefully rather than crashing.
export function decodeRecipe(encoded: string): Recipe | null {
  try {
    const parsed = JSON.parse(atob(encoded));
    if (parsed.v === 1 && Array.isArray(parsed.steps) && parsed.steps.length > 0) {
      return parsed as Recipe;
    }
  } catch {}
  return null;
}

// stepLabel produces a human-readable summary of a single step. It is used in
// two places: RecipeCard renders the pending step list before the user applies
// a recipe, and ExportMode shows the already-applied steps in the export panel.
export function stepLabel(step: RecipeStep): string {
  switch (step.op) {
    case "generateMissingCrossings":
      return `Generator: Generate missing crossings (${step.scope})`;
    case "makeAllSidewalks":
      return `Generator: Make all sidewalks${step.onlyMajor ? " (major roads only)" : ""}`;
    case "connectAllCrossings":
      return `Generator: Connect all crossing nodes${step.includeCrossingNo ? " (incl. crossing=no)" : ""}`;
    case "assumeTags":
      return `Generator: Autoset tags on one-ways (drive on ${step.driveOnLeft ? "left" : "right"})`;
    case "applyOverrides":
      return "Overrides: Apply manual overrides (local)";
    case "applyMaxspeed":
      return "Maxspeed: Enrich crossings with maxspeed";
    case "applyCrossingNodeTags":
      return "Crossing: Enrich ways with node crossing tags";
    case "setNetworkFilter": {
      const includeLabel: Record<NetworkFilterInclude, string> = {
        Everything: "Full network",
        OnlyExplicitFootways: "Only footways",
        RouteableNetwork: "Anything routeable for walking",
      };
      const deadends = step.ignore_deadends ? ", ignore dead ends" : "";
      return `Export: Network filter — ${includeLabel[step.include]}${deadends}`;
    }
  }
}
