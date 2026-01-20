import { convex } from "@turf/convex";
import { feature } from "@turf/helpers";
import { overpassQueryForPolygon } from "svelte-utils/osm";
import * as backendPkg from "../../../backend/pkg";
import { backend, refreshLoadingScreen } from "../";
import { fetchOverpass } from "./overpass";
import osm2geojson from "osm2geojson-ultra";
import type { Feature, Polygon, Geometry } from "geojson";

interface OverpassResponse {
  elements: Array<{
    type: string;
    id: number;
    [key: string]: unknown;
  }>;
}

async function fetchRelationGeometry(id: number): Promise<Geometry | null> {
  const query = `[out:json]; relation(${id}); out geom;`;
  const resp = await fetchOverpass(query);
  const osmJson: OverpassResponse = await resp.json();

  if (!osmJson.elements || osmJson.elements.length === 0) {
    throw new Error(`Relation ${id} not found`);
  }

  const relation = osmJson.elements[0];
  if (relation.type !== "relation") {
    throw new Error(`Object ${id} is not a relation`);
  }

  // Use elementId to filter for just this relation
  // With elementId, result is always a single Feature (never FeatureCollection)
  const result = osm2geojson(osmJson, {
    elementId: `relation/${id}`,
  });

  // Result can be Feature, FeatureCollection, or undefined
  // With elementId, it should always be a Feature
  if (!result || !("geometry" in result)) {
    return null;
  }

  return result.geometry;
}

function createConvexHull(relationGeometry: Geometry): Feature<Polygon> {
  const relationFeature = feature(relationGeometry);
  const convexHullFeature = convex(relationFeature);
  if (!convexHullFeature || convexHullFeature.geometry.type !== "Polygon") {
    throw new Error("Failed to create convex hull");
  }
  return convexHullFeature;
}

async function loadOsmDataWithBoundary(
  convexHull: Feature<Polygon>,
): Promise<Uint8Array> {
  const overpassUrl = overpassQueryForPolygon(convexHull);
  const osmResp = await fetchOverpass(overpassUrl);
  return await osmResp.bytes();
}

export async function loadRelationAndCreateSpeedwalk(
  relationId: number,
): Promise<void> {
  const relationGeometry = await fetchRelationGeometry(relationId);
  if (!relationGeometry) {
    throw new Error("Relation has no geometry");
  }

  await refreshLoadingScreen();
  const convexHull = createConvexHull(relationGeometry);

  await refreshLoadingScreen();
  const osmXml = await loadOsmDataWithBoundary(convexHull);

  await refreshLoadingScreen();
  backend.set(new backendPkg.Speedwalk(new Uint8Array(osmXml), convexHull));
}
