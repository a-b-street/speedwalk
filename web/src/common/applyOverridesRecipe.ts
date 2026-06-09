/**
 * Standalone helper for applying stored manual overrides (from IndexedDB) via the WASM backend.
 * Used by the recipe runner in ExportMode. Mirrors the core logic of OverridesMode's
 * applyEverythingInBoundary but without UI state.
 */
import {
  getOverrides,
  filterSegmentsInBoundary,
  filterDeletionsInBoundary,
} from "./localOverrides";
import { isValidSegment } from "./overridesSchema";

const crossingWayTags = {
  highway: "footway",
  footway: "crossing",
  crossing: "manual",
};

type BatchCrossingPayload = {
  start: { lng: number; lat: number };
  end: { lng: number; lat: number };
  tags: Record<string, string>;
  resolved?: unknown;
};

type BatchDeletionPayload = {
  wayId: number;
  node1: number;
  node2: number;
};

type ResolvedDeletionJson = {
  edges?: { way_id: number; node1: number; node2: number }[];
};

type BatchCapableBackend = {
  getBoundary(): string;
  resolveManualDeletion(
    startLng: number,
    startLat: number,
    endLng: number,
    endLat: number,
  ): string;
  editApplyManualOverridesBatch(
    crossings: BatchCrossingPayload[],
    deletions: BatchDeletionPayload[],
  ): void;
};

export async function applyStoredOverrides(backend: unknown): Promise<void> {
  const b = backend as BatchCapableBackend;
  const overrides = await getOverrides();
  const boundary = JSON.parse(b.getBoundary());

  const crossings = filterSegmentsInBoundary(
    overrides.addedCrossings,
    boundary,
  ).filter(isValidSegment);
  const deletions = filterDeletionsInBoundary(
    overrides.deletedWaySegments,
    boundary,
  );

  // Deduplicate deletions by draft geometry, then resolve to way/node IDs
  type Draft = { start: { lng: number; lat: number }; end: { lng: number; lat: number } };
  const drafts = new Map<string, Draft>();
  for (const seg of deletions) {
    if (seg.draftStart && seg.draftEnd) {
      const k = [
        seg.draftStart.lng.toFixed(7),
        seg.draftStart.lat.toFixed(7),
        seg.draftEnd.lng.toFixed(7),
        seg.draftEnd.lat.toFixed(7),
      ].join("|");
      if (!drafts.has(k)) {
        drafts.set(k, { start: seg.draftStart, end: seg.draftEnd });
      }
    }
  }

  const deletionPayloads: BatchDeletionPayload[] = [];
  if (!("resolveManualDeletion" in b)) {
    if (drafts.size > 0) {
      console.warn(
        "[Overrides] resolveManualDeletion not available in this WASM build; deletions skipped",
      );
    }
  } else {
    const seen = new Set<string>();
    for (const draft of drafts.values()) {
      const raw = JSON.parse(
        b.resolveManualDeletion(
          draft.start.lng,
          draft.start.lat,
          draft.end.lng,
          draft.end.lat,
        ),
      ) as ResolvedDeletionJson;
      for (const e of raw.edges ?? []) {
        const key = `${e.way_id}:${e.node1}:${e.node2}`;
        if (!seen.has(key)) {
          seen.add(key);
          deletionPayloads.push({ wayId: e.way_id, node1: e.node1, node2: e.node2 });
        }
      }
    }
  }

  const crossingPayloads: BatchCrossingPayload[] = crossings.map((seg) => ({
    start: seg.start,
    end: seg.end,
    tags: { ...crossingWayTags, ...seg.tags },
    resolved: seg.resolved,
  }));

  b.editApplyManualOverridesBatch(crossingPayloads, deletionPayloads);
}
