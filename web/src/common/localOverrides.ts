import type { GeoJSON } from "geojson";
import { bbox } from "svelte-utils/map";

const DB_NAME = "speedwalk-overrides";
const STORE_NAME = "regionOverrides";
const GLOBAL_KEY = "global";

export interface AddedCrossingSegment {
  id?: string;
  start: { lat: number; lng: number };
  end: { lat: number; lng: number };
  tags: Record<string, string>;
}

export interface RegionOverrides {
  version: number;
  addedCrossings: AddedCrossingSegment[];
}

const DEFAULT_OVERRIDES: RegionOverrides = {
  version: 1,
  addedCrossings: [],
};

function openDb(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, 1);
    req.onerror = () => reject(req.error);
    req.onsuccess = () => resolve(req.result);
    req.onupgradeneeded = () => {
      req.result.createObjectStore(STORE_NAME, { keyPath: "regionKey" });
    };
  });
}

/** True if the item is a segment (has start and end). Drops legacy single-point entries. */
function isSegment(
  x: AddedCrossingSegment | { lat?: number; lng?: number },
): x is AddedCrossingSegment {
  return (
    x != null &&
    "start" in x &&
    "end" in x &&
    typeof (x as AddedCrossingSegment).start?.lat === "number" &&
    typeof (x as AddedCrossingSegment).end?.lat === "number"
  );
}

export async function getOverrides(): Promise<RegionOverrides> {
  const db = await openDb();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, "readonly");
    const store = tx.objectStore(STORE_NAME);
    const req = store.get(GLOBAL_KEY);
    req.onerror = () => reject(req.error);
    req.onsuccess = () => {
      db.close();
      const raw = req.result as
        | (RegionOverrides & { regionKey?: string })
        | undefined;
      const list = raw?.addedCrossings ?? DEFAULT_OVERRIDES.addedCrossings;
      const addedCrossings = (
        Array.isArray(list) ? list.filter(isSegment) : []
      ).map((seg) => (seg.id ? seg : { ...seg, id: crypto.randomUUID() }));
      const data = raw
        ? {
            version: raw.version ?? DEFAULT_OVERRIDES.version,
            addedCrossings,
          }
        : { ...DEFAULT_OVERRIDES };
      resolve(data);
    };
  });
}

export async function saveOverrides(data: RegionOverrides): Promise<void> {
  const db = await openDb();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, "readwrite");
    const store = tx.objectStore(STORE_NAME);
    // Plain object so IndexedDB structured-clone succeeds (no Svelte proxies)
    const record = JSON.parse(
      JSON.stringify({
        regionKey: GLOBAL_KEY,
        version: data.version,
        addedCrossings: data.addedCrossings,
      }),
    );
    const req = store.put(record);
    req.onerror = () => reject(req.error);
    req.onsuccess = () => {
      db.close();
      resolve();
    };
  });
}

/** Returns segments whose midpoint is inside the boundary's bbox. */
export function filterSegmentsInRegion(
  segments: AddedCrossingSegment[],
  boundaryGeoJson: GeoJSON,
): AddedCrossingSegment[] {
  const [minLng, minLat, maxLng, maxLat] = bbox(boundaryGeoJson);
  return segments.filter((seg) => {
    const midLng = (seg.start.lng + seg.end.lng) / 2;
    const midLat = (seg.start.lat + seg.end.lat) / 2;
    return (
      midLng >= minLng &&
      midLng <= maxLng &&
      midLat >= minLat &&
      midLat <= maxLat
    );
  });
}

export async function deleteOverrides(): Promise<void> {
  const db = await openDb();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, "readwrite");
    const store = tx.objectStore(STORE_NAME);
    const req = store.delete(GLOBAL_KEY);
    req.onerror = () => reject(req.error);
    req.onsuccess = () => {
      db.close();
      resolve();
    };
  });
}
