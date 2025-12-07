<script lang="ts">
  import { backend, debugMode, map, mutationCounter } from "../";
  import {
    roadLineWidth,
    colors,
    type WayProps,
    type NodeProps,
  } from "../sidewalks";
  import type { MapMouseEvent } from "maplibre-gl";
  import {
    GeoJSON,
    CircleLayer,
    LineLayer,
    SymbolLayer,
    MapEvents,
  } from "svelte-maplibre";
  import {
    isPoint,
    isLine,
    emptyGeojson,
    constructMatchExpression,
  } from "svelte-utils/map";
  import type {
    Feature,
    LineString,
    FeatureCollection,
    Geometry,
    Point,
  } from "geojson";

  // This is meant as a deeply interactive layer

  export let pinnedWay: Feature<LineString, WayProps> | null = null;
  export let drawProblemDetails: FeatureCollection<
    Geometry,
    { label: string; color: string }
  >;
  export let showProblemDetails: boolean;

  let nodes = emptyGeojson() as FeatureCollection<Point, NodeProps>;
  let ways = emptyGeojson() as FeatureCollection<LineString, WayProps>;

  $: update($mutationCounter);
  function update(_: number) {
    nodes = JSON.parse($backend!.getNodes());
    ways = JSON.parse($backend!.getWays());

    if (pinnedWay) {
      let findId = pinnedWay.id;
      pinnedWay = ways.features.find((f) => f.id == findId)!;
    }
  }

  function onMapClick(e: CustomEvent<MapMouseEvent>) {
    pinnedWay = null;
    for (let rendered of $map!.queryRenderedFeatures(e.detail.point, {
      layers: ["ways"],
    })) {
      // Find the original feature in the GJ, to avoid having to parse nested properties
      pinnedWay = ways.features.find((f) => f.id == rendered.id)!;
      break;
    }
  }

  $: pinnedWaySides =
    $backend && pinnedWay
      ? JSON.parse($backend.getSideLocations(BigInt(pinnedWay.properties.id)))
      : emptyGeojson();

  $: drawProblemDetails = problemDetails(pinnedWay);
  function problemDetails(
    pinnedWay: Feature<LineString, WayProps> | null,
  ): FeatureCollection<Geometry, { label: string; color: string }> {
    let gj = emptyGeojson();
    if (pinnedWay) {
      for (let problem of pinnedWay.properties.problems) {
        gj.features = [...gj.features, ...problem.details];
      }
    }
    return gj as FeatureCollection<Geometry, { label: string; color: string }>;
  }

  function showNodeOrder(
    pinnedWay: Feature<LineString, WayProps> | null,
  ): FeatureCollection {
    let gj = emptyGeojson();
    if (pinnedWay) {
      for (let [idx, node] of pinnedWay.properties.node_ids.entries()) {
        let f = nodes.features.find((f) => f.properties.id == node)!;
        gj.features.push({
          type: "Feature",
          geometry: f.geometry,
          properties: { idx },
        });
      }
    }
    return gj;
  }

  function snappedRoad(
    pinnedWay: Feature<LineString, WayProps> | null,
    debugMode: boolean,
  ): FeatureCollection {
    if (!pinnedWay || !debugMode) {
      return emptyGeojson();
    }
    let find = parseInt(pinnedWay.properties.tags["tmp:closest_way"]);
    if (!find) {
      return emptyGeojson();
    }

    for (let way of ways.features) {
      if (way.properties.id == find) {
        let copy = JSON.parse(JSON.stringify(way));
        copy.properties.left = pinnedWay.properties.tags["tmp:side"] == "Left";
        return {
          type: "FeatureCollection",
          features: [copy],
        };
      }
    }
    return emptyGeojson();
  }
</script>

<MapEvents on:click={onMapClick} />

<GeoJSON data={pinnedWay || emptyGeojson()}>
  <LineLayer
    id="pinned"
    beforeId="Road labels"
    paint={{
      "line-width": roadLineWidth(10),
      "line-color": "cyan",
      "line-opacity": 0.5,
    }}
  />

  <SymbolLayer
    minzoom={12}
    layout={{
      "icon-image": "arrow",
      "icon-size": 1.0,
      "symbol-placement": "line",
      "symbol-spacing": 50,
      "icon-allow-overlap": true,
    }}
  />
</GeoJSON>

<GeoJSON data={snappedRoad(pinnedWay, $debugMode)}>
  <LineLayer
    id="snapped-to-pinned"
    beforeId="Road labels"
    paint={{
      "line-width": roadLineWidth(10),
      "line-color": "blue",
      "line-opacity": 0.5,
      "line-offset": ["case", ["get", "left"], -5, 5],
    }}
  />
</GeoJSON>

<GeoJSON data={ways}>
  <LineLayer
    id="ways"
    beforeId="Road labels"
    manageHoverState
    hoverCursor="pointer"
    paint={{
      "line-width": roadLineWidth(0),
      "line-color": constructMatchExpression(["get", "kind"], colors, "cyan"),
    }}
  />
</GeoJSON>

<GeoJSON data={pinnedWaySides}>
  <SymbolLayer
    id="pinned-sides"
    paint={{
      "text-color": "black",
      "text-halo-color": "cyan",
      "text-halo-width": 4,
    }}
    layout={{
      "text-field": ["get", "side"],
      "text-size": 16,
      "symbol-placement": "line",
    }}
  />
</GeoJSON>

<GeoJSON data={showNodeOrder(pinnedWay)}>
  <SymbolLayer
    id="pinned-node-order"
    paint={{
      "text-color": "white",
      "text-halo-color": "blue",
      "text-halo-width": 5,
    }}
    layout={{
      "text-field": ["get", "idx"],
      "text-size": 16,
      visibility: $debugMode ? "visible" : "none",
    }}
  />
</GeoJSON>

<GeoJSON data={drawProblemDetails}>
  <CircleLayer
    filter={isPoint}
    paint={{
      "circle-radius": 20,
      "circle-color": ["get", "color"],
      "circle-opacity": 0.5,
    }}
    layout={{
      visibility: showProblemDetails ? "visible" : "none",
    }}
  />

  <LineLayer
    filter={isLine}
    paint={{
      "line-width": roadLineWidth(5),
      "line-color": ["get", "color"],
      "line-opacity": 0.5,
    }}
    layout={{
      visibility: showProblemDetails ? "visible" : "none",
    }}
  />
</GeoJSON>
