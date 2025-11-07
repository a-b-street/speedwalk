<script lang="ts">
  import Edits from "./Edits.svelte";
  import BulkOperations from "./BulkOperations.svelte";
  import { backend, mutationCounter } from "../";
  import { problemTypes, colors, type NodeProps, type WayProps } from "./";
  import type {
    Map,
    MapMouseEvent,
    ExpressionSpecification,
  } from "maplibre-gl";
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
  let onlyModified = false;
  let onlySeverances = false;
  let drawProblemDetails = emptyGeojson();
  let showProblems = false;
  let showProblemTypes: Record<string, boolean> = Object.fromEntries(
    problemTypes.map((k) => [k, true]),
  );

  $: updateModel($mutationCounter);
  function updateModel(mutationCounter: number) {
    nodes = JSON.parse($backend!.getNodes());
    ways = JSON.parse($backend!.getWays());

    for (let x of [...nodes.features, ...ways.features]) {
      let types = [];
      for (let problem of x.properties.problems) {
        types.push(problem.note);
      }
      // @ts-expect-error TODO Hack for maplibre "in" expressions
      x.properties.problem_types = types;
    }

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

  function getOsmTimestamp(): string {
    let t = $backend!.getOsmTimestamp();
    if (t) {
      let d = new Date(1000 * Number(t));
      return d.toDateString();
    }
    return "unknown";
  }

  $: pinnedWaySides =
    $backend && pinnedWay
      ? JSON.parse($backend.getSideLocations(BigInt(pinnedWay.properties.id)))
      : emptyGeojson();

  $: drawProblemDetails = problemDetails(pinnedWay);
  function problemDetails(
    pinnedWay: Feature<LineString, WayProps> | null,
  ): FeatureCollection {
    let gj = emptyGeojson();
    if (pinnedWay) {
      for (let problem of pinnedWay.properties.problems) {
        gj.features = [...gj.features, ...problem.details];
      }
    }
    return gj;
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

  function filterWays(
    _a: boolean,
    _b: boolean,
    _c: boolean,
    _d: any,
  ): ExpressionSpecification {
    let all = [];
    if (onlySeverances) {
      all.push([
        "any",
        ["in", ["get", "kind"], ["literal", ["Sidewalk", "Crossing", "Other"]]],
        ["get", "is_severance"],
      ]);
    }
    if (onlyModified) {
      all.push(["get", "modified"]);
    }
    if (showProblems) {
      let any = [];
      for (let [key, value] of Object.entries(showProblemTypes)) {
        if (value) {
          any.push(["in", key, ["get", "problem_types"]]);
        }
      }
      all.push(["any", ...any]);
    }
    return ["all", ...all] as ExpressionSpecification;
  }

  function clear() {
    $backend = null;
  }
</script>

<SplitComponent>
  <div slot="sidebar">
    <button class="btn btn-secondary" on:click={clear}>
      Load another area
    </button>

    <p>OSM data is from {getOsmTimestamp()}</p>

    <Edits />

    {#if pinnedWay}
      <WayDetails {pinnedWay} {drawProblemDetails} />
    {:else}
      <BulkOperations />
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
        filter={filterWays(
          onlySeverances,
          onlyModified,
          showProblems,
          showProblemTypes,
        )}
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
            onlyModified ? ["case", ["get", "modified"], 1.0, 0.5] : 1.0,
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
          "text-color": "white",
          "text-halo-color": "blue",
          "text-halo-width": 5,
        }}
        layout={{
          "text-field": ["get", "idx"],
          "text-size": 16,
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
      />

      <LineLayer
        filter={isLine}
        paint={{
          "line-width": 20,
          "line-color": ["get", "color"],
          "line-opacity": 0.5,
        }}
      />
    </GeoJSON>

    <Control position="top-right">
      <div class="card" style:width="250px" style:padding="8px">
        <div class="card-header">Filters</div>
        <div class="card-body">
          <Checkbox bind:checked={showNodes}>Nodes</Checkbox>

          <Checkbox bind:checked={onlyModified}>Only modified objects</Checkbox>

          <Checkbox bind:checked={onlySeverances}>
            Only show severance roads
          </Checkbox>

          <Checkbox bind:checked={showProblems}>Only show problems</Checkbox>
          {#if showProblems}
            {#each problemTypes as key}
              <Checkbox bind:checked={showProblemTypes[key]}>{key}</Checkbox>
            {/each}
          {/if}

          <Metrics />
        </div>
      </div>
    </Control>
  </div>
</SplitComponent>
