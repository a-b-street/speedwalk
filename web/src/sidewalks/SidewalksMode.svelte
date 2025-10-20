<script lang="ts">
  import Edits from "../Edits.svelte";
  import { backend, mutationCounter } from "../";
  import { colors, type NodeProps, type WayProps } from "./";
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
    isPoint,
    isLine,
    emptyGeojson,
    constructMatchExpression,
  } from "svelte-utils/map";
  import { Checkbox } from "svelte-utils";
  import type { Feature, LineString, FeatureCollection, Point } from "geojson";
  import Metrics from "./Metrics.svelte";
  import WayDetails from "./WayDetails.svelte";
  //import ExtraContext from "./ExtraContext.svelte";
  import Problems from "./Problems.svelte";

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
  let drawProblems = emptyGeojson();

  let driveOnLeft = true;
  let onlyMakeSeverances = true;

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

  // TODO Move to MapContextMenu after svelte 5 upgrade
  function onRightClick(e: CustomEvent<MapMouseEvent>) {
    $backend!.editAddNewCrossing(e.detail.lngLat.lng, e.detail.lngLat.lat);
    $mutationCounter++;
  }

  function makeAllSidewalksV2() {
    $backend!.editMakeAllSidewalksV2(onlyMakeSeverances);
    $mutationCounter++;
  }

  function connectAllCrossings() {
    $backend!.editConnectAllCrossings();
    $mutationCounter++;
  }

  function assumeTags() {
    $backend!.editAssumeTags(driveOnLeft);
    $mutationCounter++;
  }

  function getOsmTimestamp(): string {
    let t = $backend!.getOsmTimestamp();
    if (t) {
      let d = new Date(1000 * Number(t));
      return d.toDateString();
    }
    return "unknown";
  }

  $: pinnedWaySides = pinnedWay
    ? JSON.parse($backend!.getSideLocations(BigInt(pinnedWay.properties.id)))
    : emptyGeojson();

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
  ): FeatureCollection {
    if (!pinnedWay) {
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

<SplitComponent>
  <div slot="sidebar">
    <p>OSM data is from {getOsmTimestamp()}</p>

    <Edits />

    {#if pinnedWay}
      <WayDetails {pinnedWay} />
    {/if}

    <div class="card">
      <div class="card-header">Bulk operations</div>
      <div class="card-body">
        <div class="card mb-3">
          <div class="card-header">Assume old-style tags on one-ways</div>
          <div class="card-body">
            <Checkbox bind:checked={driveOnLeft}>Drive on the left</Checkbox>
            <button class="btn btn-secondary" on:click={assumeTags}>
              Autoset tags on one-ways
            </button>
          </div>
        </div>

        <div class="card mb-3">
          <div class="card-header">Make all sidewalks</div>
          <div class="card-body">
            <Checkbox bind:checked={onlyMakeSeverances}>
              Only for severances
            </Checkbox>
            <button class="btn btn-secondary" on:click={makeAllSidewalksV2}>
              Make sidewalks
            </button>
          </div>
        </div>

        <button class="btn btn-secondary" on:click={connectAllCrossings}>
          Connect all crossings over severances
        </button>
      </div>
    </div>

    <Problems bind:map bind:drawProblems />
  </div>

  <div slot="map">
    <MapEvents on:click={onMapClick} on:contextmenu={onRightClick} />

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

    <GeoJSON data={snappedRoad(pinnedWay)}>
      <LineLayer
        id="snapped-to-pinned"
        beforeId="Road labels"
        paint={{
          "line-width": 15,
          "line-color": "blue",
          "line-opacity": 0.5,
          "line-offset": ["case", ["get", "left"], -3, 3],
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
        <Popup openOn="hover" let:data>
          {@const props = data?.properties ?? {}}
          <h4>Node {props.id}</h4>
          <p>Ways: {props.way_ids}</p>
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
          "text-color": "black",
          "text-halo-color": "red",
          "text-halo-width": 4,
        }}
        layout={{
          "text-field": ["get", "idx"],
          "text-size": 16,
        }}
      />
    </GeoJSON>

    <GeoJSON data={drawProblems}>
      <CircleLayer
        filter={isPoint}
        paint={{
          "circle-radius": 20,
          "circle-color": "yellow",
          "circle-opacity": 0.5,
        }}
      />

      <LineLayer
        filter={isLine}
        paint={{
          "line-width": 20,
          "line-color": "yellow",
          "line-opacity": 0.5,
        }}
      />
    </GeoJSON>

    <!--<ExtraContext show={showExtraContext} />-->

    <Control position="top-right">
      <div style:background="white" style:width="200px" style:padding="8px">
        <Checkbox bind:checked={showNodes}>Nodes</Checkbox>

        <Checkbox bind:checked={showExtraContext}>Extra context</Checkbox>

        <Checkbox bind:checked={fadeUnmodified}>Fade unmodified ways</Checkbox>

        <Metrics />
      </div>
    </Control>
  </div>
</SplitComponent>
