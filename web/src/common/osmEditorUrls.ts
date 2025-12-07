import type { Map } from "maplibre-gl";

export function getMapViewport(map: Map | null): {
  lat: number;
  lng: number;
  zoom: number;
} | null {
  if (!map) return null;
  const center = map.getCenter();
  return {
    lat: center.lat,
    lng: center.lng,
    zoom: map.getZoom(),
  };
}

export function getOsmUrl(zoom: number, lat: number, lng: number): string {
  const hashParams = new URLSearchParams();
  hashParams.append(
    "map",
    `${Math.round(zoom)}/${lat.toFixed(6)}/${lng.toFixed(6)}`,
  );
  return `https://www.openstreetmap.org/#${hashParams.toString()}`;
}

export function getIdUrl(zoom: number, lat: number, lng: number): string {
  const hashParams = new URLSearchParams();
  hashParams.append(
    "map",
    `${Math.round(zoom)}/${lat.toFixed(6)}/${lng.toFixed(6)}`,
  );
  return `https://www.openstreetmap.org/edit?editor=id#${hashParams.toString()}`;
}

export function getKiwidUrl(zoom: number, lat: number, lng: number): string {
  const hashParams = new URLSearchParams();
  hashParams.append(
    "map",
    `${Math.round(zoom)}/${lat.toFixed(6)}/${lng.toFixed(6)}`,
  );
  hashParams.append("disable_features", "boundaries");
  hashParams.append("hashtags", "Speedwalk");
  return `https://kyle.kiwi/iD/#${hashParams.toString()}`;
}

export function getRapidUrl(zoom: number, lat: number, lng: number): string {
  const hashParams = new URLSearchParams();
  hashParams.append(
    "map",
    `${Math.round(zoom)}/${lat.toFixed(6)}/${lng.toFixed(6)}`,
  );
  hashParams.append("disable_features", "boundaries");
  hashParams.append("hashtags", "Speedwalk");
  return `https://rapideditor.org/edit#${hashParams.toString()}`;
}

// Requires JOSM to be running with remote control enabled
export function getJosmUrlFromBounds(
  left: number,
  right: number,
  top: number,
  bottom: number,
): string {
  return `http://127.0.0.1:8111/load_and_zoom?left=${left.toFixed(6)}&right=${right.toFixed(6)}&top=${top.toFixed(6)}&bottom=${bottom.toFixed(6)}`;
}

export function getJosmUrlFromMap(map: Map): string {
  const bounds = map.getBounds();
  return getJosmUrlFromBounds(
    bounds.getWest(),
    bounds.getEast(),
    bounds.getNorth(),
    bounds.getSouth(),
  );
}
