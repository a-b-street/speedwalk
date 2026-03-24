import * as z from "zod/mini";

const pointSchema = z.object({ lat: z.number(), lng: z.number() });

export const snappedSegmentSchema = z.object({
  start: pointSchema,
  end: pointSchema,
});

export const resolvedCrossingSegmentSchema = z.object({
  startWay: z.number(),
  endWay: z.number(),
  start: pointSchema,
  end: pointSchema,
});

const addedCrossingSegmentSchema = z.extend(snappedSegmentSchema, {
  id: z.optional(z.string()),
  tags: z._default(z.optional(z.record(z.string(), z.string())), {}),
  resolved: z.optional(resolvedCrossingSegmentSchema),
});

export type SnappedSegment = {
  start: { lat: number; lng: number };
  end: { lat: number; lng: number };
};

export type AddedCrossingSegment = SnappedSegment & {
  id?: string;
  tags: Record<string, string>;
  resolved?: ResolvedCrossingSegment;
};

export type ResolvedCrossingSegment = {
  startWay: number;
  endWay: number;
  start: { lat: number; lng: number };
  end: { lat: number; lng: number };
};

export const deletedWaySegmentSchema = z.object({
  id: z.optional(z.string()),
  wayId: z.number(),
  node1: z.number(),
  node2: z.number(),
  node1Lat: z.optional(z.number()),
  node1Lng: z.optional(z.number()),
  node2Lat: z.optional(z.number()),
  node2Lng: z.optional(z.number()),
  midLat: z.number(),
  midLng: z.number(),
  draftStart: z.optional(pointSchema),
  draftEnd: z.optional(pointSchema),
  tags: z._default(z.optional(z.record(z.string(), z.string())), {}),
});

export type DeletedWaySegment = {
  id?: string;
  wayId: number;
  node1: number;
  node2: number;
  node1Lat?: number;
  node1Lng?: number;
  node2Lat?: number;
  node2Lng?: number;
  midLat: number;
  midLng: number;
  draftStart?: { lat: number; lng: number };
  draftEnd?: { lat: number; lng: number };
  tags: Record<string, string>;
};

export const manualOverridesSchema = z.object({
  version: z.number(),
  addedCrossings: z.array(addedCrossingSegmentSchema),
  deletedWaySegments: z._default(z.array(deletedWaySegmentSchema), []),
});

export type ManualOverrides = {
  version: number;
  addedCrossings: AddedCrossingSegment[];
  deletedWaySegments: DeletedWaySegment[];
};

export function isValidSegment(x: unknown): x is AddedCrossingSegment {
  return addedCrossingSegmentSchema.safeParse(x).success;
}

export function isValidDeletedSegment(x: unknown): x is DeletedWaySegment {
  return deletedWaySegmentSchema.safeParse(x).success;
}
