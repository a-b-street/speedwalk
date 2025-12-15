import { useQueryState, createParser } from "nuqs-svelte";

export const roundNumber = (number: number, precision: number) => {
  return Number.parseFloat(number.toFixed(precision));
};

export type MapViewport = {
  zoom: number;
  lat: number;
  lng: number;
};

const mapViewportParser = createParser<MapViewport>({
  parse(query: string) {
    const parts = query.split("/");
    if (parts.length !== 3) {
      return null;
    }
    const zoom = Number.parseFloat(parts[0]);
    const lat = Number.parseFloat(parts[1]);
    const lng = Number.parseFloat(parts[2]);
    if (
      isNaN(zoom) ||
      isNaN(lat) ||
      isNaN(lng) ||
      zoom < 0 ||
      lat < -90 ||
      lat > 90 ||
      lng < -180 ||
      lng > 180
    ) {
      return null;
    }
    return { zoom, lat, lng };
  },
  serialize(value: MapViewport) {
    const zoom = roundNumber(value.zoom, 2);
    const lat = roundNumber(value.lat, 6);
    const lng = roundNumber(value.lng, 6);
    return `${zoom}/${lat}/${lng}`;
  },
});

export function useMapViewport() {
  return useQueryState(
    "map",
    mapViewportParser.withOptions({
      history: "replace",
    }),
  );
}
