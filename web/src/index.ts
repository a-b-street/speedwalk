import { type Writable, writable } from "svelte/store";
import { Speedwalk } from "backend";

export type Mode = "sidewalks";

export let backend: Writable<Speedwalk | null> = writable(null);
export let mutationCounter = writable(0);
export let mode: Writable<Mode> = writable("sidewalks");

export function sum(list: number[]): number {
  return list.reduce((total, x) => total + x, 0);
}

export function prettyPrintDistance(meters: number): string {
  if (meters < 1000.0) {
    return Math.round(meters) + "m";
  }
  return (meters / 1000.0).toFixed(1) + "km";
}
