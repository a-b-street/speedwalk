import { type Writable, writable } from "svelte/store";
import * as backendPkg from "../../backend/pkg";
import type { Map } from "maplibre-gl";
import { basemapStyles } from "svelte-utils/map";
import { localStorageStore } from "svelte-utils";

export let map: Writable<Map | null> = writable(null);
export let backend: Writable<backendPkg.Speedwalk | null> = writable(null);
export let mutationCounter = writable(0);
export let anyEdits = writable(false);

export let loggedInUser: Writable<
  { name: string; uid: number; avatarUrl: string } | undefined
> = writable();

export type Mode =
  | { kind: "sidewalks" }
  | { kind: "crossings" }
  | { kind: "disconnections" }
  | { kind: "generator" }
  | { kind: "export" }
  | { kind: "overrides" }
  | { kind: "maxspeed" };

/** Mode kinds only available when use case is "route-networks". When switching to audit, redirect off these. */
export const ROUTE_NETWORK_ONLY_MODE_KINDS: Mode["kind"][] = [
  "generator",
  "overrides",
  "export",
  "maxspeed",
];

export const DEFAULT_AUDIT_MODE: Mode = { kind: "sidewalks" };

export let mode: Writable<Mode> = writable({ kind: "sidewalks" });

export type UseCase = "audit" | "route-networks";
export let useCase: Writable<UseCase> = localStorageStore(
  "speedwalk-useCase",
  "audit",
);

export let debugMode = writable(false);

const defaultNetworkFilter = {
  include: "RouteableNetwork" as const,
  ignore_deadends: true,
};
export let networkFilter = localStorageStore(
  "speedwalk-networkFilter",
  defaultNetworkFilter,
);

export let onlyMajorRoadsBulk = localStorageStore(
  "speedwalk-onlyMajorRoads",
  true,
);
export let includeCrossingNoBulk = localStorageStore(
  "speedwalk-includeCrossingNo",
  false,
);

export type CrossingScopeBulk = "major" | "minor" | "all";
export let crossingScopeBulk = localStorageStore<CrossingScopeBulk>(
  "speedwalk-crossingScopeBulk",
  "major",
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

// Store the original keys
export let originalBasemapStyles = new Set(basemapStyles.keys());
