<script lang="ts">
  import { backend, mutationCounter } from "../";
  import { previewSidewalk, colors, type NodeProps, type WayProps } from "./";
  import type { Map, MapMouseEvent } from "maplibre-gl";
  import {
    GeoJSON,
    CircleLayer,
    LineLayer,
    hoverStateFilter,
    SymbolLayer,
    MapEvents,
    Control,
    Popup,
  } from "svelte-maplibre";
  import { SplitComponent } from "svelte-utils/two_column_layout";
  import {
    emptyGeojson,
    constructMatchExpression,
    isLine,
    isPoint,
  } from "svelte-utils/map";
  import type { Feature, LineString, FeatureCollection, Point } from "geojson";
  import Metrics from "./Metrics.svelte";
  import WayDetails from "./WayDetails.svelte";
  import ExtraContext from "./ExtraContext.svelte";

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
  let showExtraContext = false;
  let fadeUnmodified = false;

  let trimBackFromCrossings = 0.0;
  let assumeBothForMissing = false;
  let onlySeverances = false;

  $: updateModel($mutationCounter);
  function updateModel(mutationCounter: number) {
    nodes = JSON.parse($backend!.getNodes());
    ways = JSON.parse($backend!.getWays());

    if (pinnedWay) {
      let findId = pinnedWay.id;
      pinnedWay = ways.features.find((f) => f.id == findId)!;
    }
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

  function hasTags(features: Feature[]): boolean {
    return features[0]?.properties?.tags;
  }

  function makeAllSidewalks() {
    console.time("makeAllSidewalks");
    $backend!.editMakeAllSidewalks(
      trimBackFromCrossings > 0 ? trimBackFromCrossings : null,
      assumeBothForMissing,
      onlySeverances,
    );
    console.timeEnd("makeAllSidewalks");
    $mutationCounter++;
  }

  function connectAllCrossings() {
    console.time("connectAllCrossings");
    $backend!.editConnectAllCrossings();
    console.timeEnd("connectAllCrossings");
    $mutationCounter++;
  }

  $: if (!pinnedWay) {
    $previewSidewalk = null;
  }

  $: pinnedWaySides = pinnedWay
    ? JSON.parse($backend!.getSideLocations(BigInt(pinnedWay.properties.id)))
    : emptyGeojson();
</script>

<SplitComponent>
  <div slot="sidebar">
    {#if pinnedWay}
      <WayDetails {pinnedWay} {trimBackFromCrossings} />
    {/if}

    <label>
      Trim back from crossings (0 means make new side road crossings)
      <input
        type="number"
        bind:value={trimBackFromCrossings}
        min="0"
        max="5"
        step="0.5"
      />
    </label>

    <details>
      <summary>Make all sidewalks</summary>

      <label>
        <input type="checkbox" bind:checked={assumeBothForMissing} />
        When sidewalk tag missing, assume both?
      </label>

      <label>
        <input type="checkbox" bind:checked={onlySeverances} />
        Only generate along severances
      </label>

      <button class="secondary" on:click={makeAllSidewalks}>
        Make all sidewalks
      </button>

      <button class="secondary" on:click={connectAllCrossings}>
        Connect all crossings over severances
      </button>
    </details>
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
          "line-opacity": fadeUnmodified
            ? ["case", ["get", "modified"], 1.0, 0.5]
            : 1.0,
        }}
      />
    </GeoJSON>

    <GeoJSON data={$previewSidewalk || emptyGeojson()}>
      <LineLayer
        filter={isLine}
        id="preview-sidewalk"
        beforeId="Road labels"
        paint={{
          "line-width": 5,
          "line-color": "purple",
        }}
      />

      <CircleLayer
        filter={isPoint}
        id="preview-sidewalk-new-nodes"
        beforeId="Road labels"
        paint={{
          "circle-radius": 7,
          "circle-color": "yellow",
          "circle-stroke-color": "black",
          "circle-stroke-width": 1,
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
          "circle-color": ["case", ["get", "is_crossing"], "yellow", "grey"],
          "circle-opacity": [
            "case",
            ["boolean", ["get", "is_crossing"]],
            fadeUnmodified ? ["case", ["get", "modified"], 1.0, 0.5] : 1.0,
            0,
          ],
          "circle-stroke-color": ["case", ["has", "tags"], "black", "grey"],
          "circle-stroke-width": 1,
        }}
        layout={{
          visibility: showNodes ? "visible" : "none",
        }}
      >
        <Popup openOn="hover" let:data canOpen={hasTags}>
          {@const props = data?.properties ?? {}}
          <h4>Node {props.id}</h4>
          <table>
            {#each Object.entries(JSON.parse(props.tags || "{}")) as [key, value]}
              <tr>
                <td>{key}</td>
                <td>{value}</td>
              </tr>
            {/each}
          </table>
        </Popup>
      </CircleLayer>

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
    </GeoJSON>

    <ExtraContext show={showExtraContext} />

    <Control position="top-right">
      <div style:background="white" style:width="200px" style:padding="8px">
        <label>
          <input type="checkbox" bind:checked={showNodes} />
          Nodes
        </label>

        <label>
          <input type="checkbox" bind:checked={showExtraContext} />
          Extra context
        </label>

        <label>
          <input type="checkbox" bind:checked={fadeUnmodified} />
          Fade unmodified ways
        </label>

        <Metrics />
      </div>
    </Control>
  </div>
</SplitComponent>
