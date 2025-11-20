import { type Writable, writable } from "svelte/store";
import * as backendPkg from "../../backend/pkg";
import type { ExpressionSpecification } from "maplibre-gl";

export let backend: Writable<backendPkg.Speedwalk | null> = writable(null);
export let mutationCounter = writable(0);

export let loggedInUser: Writable<
  { name: string; uid: number; avatarUrl: string } | undefined
> = writable();

export let enabledBulkOps = writable(false);

export function sum(list: number[]): number {
  return list.reduce((total, x) => total + x, 0);
}

export function prettyPrintDistance(meters: number): string {
  if (meters < 1000.0) {
    return Math.round(meters) + " m";
  }
  return (meters / 1000.0).toFixed(1) + " km";
}

// This is a replacement for `svelte.tick`, which doesn't seem to work for some
// reason. Wait for two frames, to give the Loading component a chance to
// update, before doing someting blocking on the UI thread.
// TODO Upstream?
export async function refreshLoadingScreen(): Promise<void> {
  await new Promise((resolve) => {
    requestAnimationFrame(() => {
      requestAnimationFrame(resolve);
    });
  });
}

// Zoom-dependant line width, adapted from from the Minor road layer (secondary road class) from
// https://api.maptiler.com/maps/streets-v2/style.json. At high zoom, make sidewalks much thinner.
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
    [
      "case",
      ["==", "Sidewalk", ["get", "kind"]],
      4 + extraWidth,
      7 + extraWidth,
    ],
    20,
    [
      "case",
      ["==", "Sidewalk", ["get", "kind"]],
      7 + extraWidth,
      24 + extraWidth,
    ],
  ];
}
