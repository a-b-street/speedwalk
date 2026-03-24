/**
 * Manual overrides (e.g. added crossing segments) stored in IndexedDB.
 * We store a single overrides blob (one record). Boundary filtering is separate:
 * filterSegmentsInBoundary() filters segments by the loaded map boundary (bbox).
 */
import type { GeoJSON } from "geojson";
import { bbox } from "svelte-utils/map";
import {
  type AddedCrossingSegment,
  type DeletedWaySegment,
  type ManualOverrides,
  isValidDeletedSegment,
  isValidSegment,
} from "./overridesSchema";

const DB_NAME = "speedwalk-overrides";
const DB_VERSION = 3;
const STORE_NAME = "overrides";
const OVERRIDES_RECORD_ID = "default";

const DEFAULT_OVERRIDES: ManualOverrides = {
  version: 1,
  addedCrossings: [],
  deletedWaySegments: [],
};

function openDb(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, DB_VERSION);
    req.onerror = () => reject(req.error);
    req.onsuccess = () => resolve(req.result);
    req.onupgradeneeded = (event) => {
      const db = (event.target as IDBOpenDBRequest).result;
      // Create store for new DB (oldVersion 0) or when upgrading to v2 (e.g. from v1). No data migration.
      if (event.oldVersion < 2) {
        db.createObjectStore(STORE_NAME, { keyPath: "id" });
      }
      // v3: same schema; added deletedWaySegments in JSON payload (handled in getOverrides).
    };
  });
}

export async function getOverrides(): Promise<ManualOverrides> {
  const db = await openDb();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, "readonly");
    const store = tx.objectStore(STORE_NAME);
    const req = store.get(OVERRIDES_RECORD_ID);
    req.onerror = () => reject(req.error);
    req.onsuccess = () => {
      const raw = req.result as (ManualOverrides & { id?: string }) | undefined;
      db.close();
      const list = raw?.addedCrossings ?? DEFAULT_OVERRIDES.addedCrossings;
      const addedCrossings = (
        Array.isArray(list) ? list.filter(isValidSegment) : []
      ).map((seg) => (seg.id ? seg : { ...seg, id: crypto.randomUUID() }));
      const delList = raw?.deletedWaySegments ?? DEFAULT_OVERRIDES.deletedWaySegments;
      const normalizedDeletedWaySegments = (
        Array.isArray(delList) ? delList.filter(isValidDeletedSegment) : []
      ).map((seg) => (seg.id ? seg : { ...seg, id: crypto.randomUUID() }));
      const droppedLegacyDeletions = normalizedDeletedWaySegments.filter(
        (seg) => !seg.draftStart || !seg.draftEnd,
      ).length;
      if (droppedLegacyDeletions > 0) {
        console.warn(
          `[Overrides] Dropping ${droppedLegacyDeletions} legacy deletion override(s) without draft points; redraw them to reapply reliably.`,
        );
      }
      const deletedWaySegments = normalizedDeletedWaySegments.filter(
        (seg) => seg.draftStart && seg.draftEnd,
      );
      const data = raw
        ? {
            version: raw.version ?? DEFAULT_OVERRIDES.version,
            addedCrossings,
            deletedWaySegments,
          }
        : { ...DEFAULT_OVERRIDES };
      resolve(data);
    };
  });
}

export async function saveOverrides(data: ManualOverrides): Promise<void> {
  const db = await openDb();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, "readwrite");
    const store = tx.objectStore(STORE_NAME);
    const record = JSON.parse(
      JSON.stringify({
        id: OVERRIDES_RECORD_ID,
        version: data.version,
        addedCrossings: data.addedCrossings,
        deletedWaySegments: data.deletedWaySegments,
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

/** Returns segments whose midpoint is inside the boundary's bbox (e.g. loaded map area). Skips invalid segments. */
export function filterSegmentsInBoundary(
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

/** Returns deleted way segments whose midpoint is inside the boundary's bbox. */
export function filterDeletionsInBoundary(
  segments: DeletedWaySegment[],
  boundaryGeoJson: GeoJSON,
): DeletedWaySegment[] {
  const [minLng, minLat, maxLng, maxLat] = bbox(boundaryGeoJson);
  return segments.filter((seg) => {
    return (
      seg.midLng >= minLng &&
      seg.midLng <= maxLng &&
      seg.midLat >= minLat &&
      seg.midLat <= maxLat
    );
  });
}
