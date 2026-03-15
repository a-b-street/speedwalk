import * as z from "zod/mini";

const pointSchema = z.object({ lat: z.number(), lng: z.number() });

export const snappedSegmentSchema = z.object({
  start: pointSchema,
  end: pointSchema,
});

const addedCrossingSegmentSchema = z.extend(snappedSegmentSchema, {
  id: z.optional(z.string()),
  tags: z._default(z.optional(z.record(z.string(), z.string())), {}),
});

export type SnappedSegment = {
  start: { lat: number; lng: number };
  end: { lat: number; lng: number };
};

export type AddedCrossingSegment = SnappedSegment & {
  id?: string;
  tags: Record<string, string>;
};

export const manualOverridesSchema = z.object({
  version: z.number(),
  addedCrossings: z.array(addedCrossingSegmentSchema),
});

export type ManualOverrides = {
  version: number;
  addedCrossings: AddedCrossingSegment[];
};

export function isValidSegment(x: unknown): x is AddedCrossingSegment {
  return addedCrossingSegmentSchema.safeParse(x).success;
}
