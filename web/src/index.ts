export let colors = {
  sidewalk: "black",
  good_roadway: "green",
  quickfix_roadway: "pink",
  bad_roadway: "red",
  other: "grey",
};

export function sum(list: number[]): number {
  return list.reduce((total, x) => total + x, 0);
}

export function prettyPrintDistance(meters: number): string {
  if (meters < 1000.0) {
    return Math.round(meters) + "m";
  }
  return (meters / 1000.0).toFixed(1) + "km";
}
