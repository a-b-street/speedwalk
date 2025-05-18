<script lang="ts">
  import {
    backend,
    previewSidewalk,
    colors,
    type NodeProps,
    type WayProps,
  } from "./";
  import type { Map, MapMouseEvent } from "maplibre-gl";
  import {
    GeoJSON,
    CircleLayer,
    LineLayer,
    hoverStateFilter,
    MapEvents,
    Control,
  } from "svelte-maplibre";
  import { SplitComponent } from "svelte-utils/two_column_layout";
  import {
    emptyGeojson,
    bbox,
    constructMatchExpression,
    Popup,
  } from "svelte-utils/map";
  import type { Feature, LineString, FeatureCollection, Point } from "geojson";
  import Metrics from "./Metrics.svelte";
  import WayDetails from "./WayDetails.svelte";

  export let map: Map;

  let nodes: FeatureCollection<Point, NodeProps> = JSON.parse(
    $backend!.getNodes(),
  );
  let ways: FeatureCollection<LineString, WayProps> = JSON.parse(
    $backend!.getWays(),
  );
  let pinnedWay: Feature<LineString, WayProps> | null = null;

  zoomFit();

  function zoomFit() {
    map.fitBounds(bbox(ways), {
      animate: false,
      padding: 10,
    });
  }

  function onMapClick(e: CustomEvent<MapMouseEvent>) {
    pinnedWay = null;
    for (let rendered of map.queryRenderedFeatures(e.detail.point, {
      layers: ["ways"],
    })) {
      // Find the original feature in the GJ, to avoid having to parse nested properties
      pinnedWay = ways.features.find((f) => f.id == rendered.id)!;
      break;
    }
  }

  $: if (!pinnedWay) {
    $previewSidewalk = null;
  }
</script>

<SplitComponent>
  <div slot="sidebar">
    {#if pinnedWay}
      <WayDetails {pinnedWay} />
    {/if}
  </div>

  <div slot="map">
    <MapEvents on:click={onMapClick} />

    <GeoJSON data={pinnedWay || emptyGeojson()}>
      <LineLayer
        id="pinned"
        beforeId="Road labels"
        paint={{
          "line-width": 12,
          "line-color": "cyan",
          "line-opacity": 0.5,
        }}
      />
    </GeoJSON>

    <GeoJSON data={ways}>
      <LineLayer
        id="ways"
        beforeId="Road labels"
        manageHoverState
        paint={{
          "line-width": hoverStateFilter(5, 8),
          "line-color": constructMatchExpression(
            ["get", "kind"],
            colors,
            "cyan",
          ),
        }}
      />
    </GeoJSON>

    <GeoJSON data={$previewSidewalk || emptyGeojson()}>
      <LineLayer
        id="preview-sidewalk"
        beforeId="Road labels"
        paint={{
          "line-width": 5,
          "line-color": "purple",
        }}
      />
    </GeoJSON>

    <GeoJSON data={nodes}>
      <CircleLayer
        id="nodes"
        beforeId="Road labels"
        manageHoverState
        paint={{
          "circle-radius": 7,
          "circle-color": "grey",
          "circle-opacity": hoverStateFilter(0, 0.5),
          "circle-stroke-color": "black",
          "circle-stroke-width": 1,
        }}
      >
        <Popup openOn="hover" let:props>
          <p>{JSON.stringify(props)}</p>
        </Popup>
      </CircleLayer>
    </GeoJSON>

    <Control position="top-right">
      <div style:background="white" style:width="150px">
        <Metrics />
      </div>
    </Control>
  </div>
</SplitComponent>
