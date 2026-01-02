<script lang="ts">
  import { backend, debugMode, map, mutationCounter } from "../";
  import { roadLineWidth, colors, type WayProps, type NodeProps } from "./";
  import type { MapMouseEvent, ExpressionSpecification } from "maplibre-gl";
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

  let {
    pinnedWay = $bindable(),
    drawProblemDetails = $bindable(),
    showProblemDetails,
    showRoadSides,
    nodes,
    ways,
    filterWays,
  }: {
    pinnedWay: Feature<LineString, WayProps> | null;
    drawProblemDetails: FeatureCollection<
      Geometry,
      { label: string; color: string }
    >;
    showProblemDetails: boolean;
    showRoadSides: boolean;
    nodes: FeatureCollection<Point, NodeProps>;
    ways: FeatureCollection<LineString, WayProps>;
    filterWays: ExpressionSpecification;
  } = $props();

  function onMapClick(e: MapMouseEvent) {
    pinnedWay = null;
    for (let rendered of $map!.queryRenderedFeatures(e.point, {
      layers: ["speedwalk-ways"],
    })) {
      // Find the original feature in the GJ, to avoid having to parse nested properties
      pinnedWay = ways.features.find((f) => f.id == rendered.id)!;
      break;
    }
  }

  let pinnedWaySides = $derived(
    $backend && pinnedWay
      ? JSON.parse($backend.getSideLocations(BigInt(pinnedWay.properties.id)))
      : emptyGeojson(),
  );

  // Use an effect to update a bindable
  $effect(() => {
    let gj = emptyGeojson();
    if (pinnedWay) {
      for (let problem of pinnedWay.properties.problems) {
        gj.features = [...gj.features, ...problem.details];
      }
    }
    drawProblemDetails = gj as FeatureCollection<
      Geometry,
      { label: string; color: string }
    >;
  });

  // Note these take the dependency explicitly for clarity, but also implicitly depend on things
  // like nodes
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

<MapEvents onclick={onMapClick} />

<GeoJSON data={pinnedWay || emptyGeojson()}>
  <LineLayer
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
    id="speedwalk-ways"
    beforeId="Road labels"
    manageHoverState
    hoverCursor="pointer"
    filter={filterWays}
    paint={{
      "line-width": roadLineWidth(0),
      "line-color": constructMatchExpression(["get", "kind"], colors, "cyan"),
    }}
  />
</GeoJSON>

{#key $mutationCounter}
  <GeoJSON
    data={$backend ? JSON.parse($backend.getRoadSides()) : emptyGeojson()}
  >
    <SymbolLayer
      minzoom={16}
      paint={{
        "text-color": "white",
        "text-halo-color": "black",
        "text-halo-width": 2,
      }}
      layout={{
        visibility: showRoadSides ? "visible" : "none",
        "text-field": ["get", "sidewalks"],
        "text-size": 15,
        "symbol-placement": "line",
        "symbol-spacing": 30,
        "text-allow-overlap": true,
        "text-rotation-alignment": "viewport",
      }}
    />
  </GeoJSON>
{/key}

<GeoJSON data={pinnedWaySides}>
  <SymbolLayer
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
