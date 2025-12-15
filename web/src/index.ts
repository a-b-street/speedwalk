import { type Writable, writable } from "svelte/store";
import * as backendPkg from "../../backend/pkg";
import { basemapStyles as originalBasemapStyles } from "svelte-utils/map";
import type { Map } from "maplibre-gl";

export let map: Writable<Map | null> = writable(null);
export let backend: Writable<backendPkg.Speedwalk | null> = writable(null);
export let mutationCounter = writable(0);

export let loggedInUser: Writable<
  { name: string; uid: number; avatarUrl: string } | undefined
> = writable();

export let enabledBulkOps = writable(false);
export let debugMode = writable(false);

export let networkFilter = writable<{
  include: "Everything" | "OnlyExplicitFootways" | "RouteableNetwork";
  ignore_deadends: boolean;
}>({
  include: "OnlyExplicitFootways",
  ignore_deadends: false,
});

// TODO In svelte 4, it's simplest to copy this into a store instead of mutate an import
export let basemapStyles = writable(
  JSON.parse(JSON.stringify(originalBasemapStyles)),
);

// TODO Upstream several of these
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
