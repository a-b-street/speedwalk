<script lang="ts">
  import Edits from "./Edits.svelte";
  import {
    backend,
    previewSidewalk,
    colors,
    mutationCounter,
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

  let nodes: FeatureCollection<Point, NodeProps> = {
    type: "FeatureCollection",
    features: [],
  };
  let ways: FeatureCollection<LineString, WayProps> = {
    type: "FeatureCollection",
    features: [],
  };
  let pinnedWay: Feature<LineString, WayProps> | null = null;
  let showNodes = false;
  let first = true;

  $: updateModel($mutationCounter);
  function updateModel(mutationCounter: number) {
    nodes = JSON.parse($backend!.getNodes());
    ways = JSON.parse($backend!.getWays());

    if (pinnedWay) {
      let findId = pinnedWay.id;
      pinnedWay = ways.features.find((f) => f.id == findId)!;
    }

    if (first) {
      first = false;
      zoomFit();
    }
  }

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
    <Edits />

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
        eventsIfTopMost
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
        layout={{
          visibility: showNodes ? "visible" : "none",
        }}
      >
        <Popup openOn="hover" let:props>
          <h4>Node {props.id}</h4>
          <table>
            {#each Object.entries(JSON.parse(props.tags)) as [key, value]}
              <tr>
                <td>{key}</td>
                <td>{value}</td>
              </tr>
            {/each}
          </table>
        </Popup>
      </CircleLayer>
    </GeoJSON>

    <Control position="top-right">
      <div style:background="white" style:width="200px" style:padding="8px">
        <label>
          <input type="checkbox" bind:checked={showNodes} />
          Nodes
        </label>
        <Metrics />
      </div>
    </Control>
  </div>
</SplitComponent>
