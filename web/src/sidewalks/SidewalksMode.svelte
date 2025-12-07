<script lang="ts">
  import ProblemControls from "./ProblemControls.svelte";
  import ProblemLayer from "./ProblemLayer.svelte";
  import CollapsibleCard from "../common/CollapsibleCard.svelte";
  import OsmProvenance from "./OsmProvenance.svelte";
  import Edits from "./Edits.svelte";
  import BulkOperations from "./BulkOperations.svelte";
  import {
    backend,
    mutationCounter,
    refreshLoadingScreen,
    debugMode,
    map,
  } from "../";
  import { roadLineWidth, colors, type NodeProps, type WayProps } from "./";
  import type { MapMouseEvent, ExpressionSpecification } from "maplibre-gl";
  import {
    GeoJSON,
    CircleLayer,
    LineLayer,
    SymbolLayer,
    MapEvents,
    Control,
    Popup,
  } from "svelte-maplibre";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import {
    isPoint,
    isLine,
    emptyGeojson,
    constructMatchExpression,
  } from "svelte-utils/map";
  import { Checkbox, Loading } from "svelte-utils";
  import type {
    Feature,
    LineString,
    FeatureCollection,
    Geometry,
    Point,
  } from "geojson";
  import Metrics from "./Metrics.svelte";
  import WayDetails from "./WayDetails.svelte";

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
  let showServiceRoads = true;

  let showKinds = Object.fromEntries(
    Object.keys(colors).map((kind) => [kind, true]),
  );

  let showProblemDetails = true;
  let drawProblemDetails = emptyGeojson() as FeatureCollection<
    Geometry,
    { label: string; color: string }
  >;

  let anyEdits = false;

  let loading = "";

  let drawProblems = emptyGeojson();

  $: updateModel($mutationCounter);
  async function updateModel(mutationCounter: number) {
    loading = "Recalculating model";
    await refreshLoadingScreen();
    try {
      nodes = JSON.parse($backend!.getNodes());
      ways = JSON.parse($backend!.getWays());
    } finally {
      loading = "";
    }

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
    if (!showServiceRoads) {
      all.push(["!", ["get", "is_service"]]);
    }
    if (onlyModified) {
      all.push(["get", "modified"]);
    }
    all.push([
      "in",
      ["get", "kind"],
      [
        "literal",
        Object.entries(showKinds)
          .filter((pair) => pair[1])
          .map((pair) => pair[0]),
      ],
    ]);
    return ["all", ...all] as ExpressionSpecification;
  }

  function clear() {
    if (
      anyEdits &&
      !window.confirm(
        "Changing areas will discard your current edits. Do you want to clear the edits?",
      )
    ) {
      return;
    }
    $backend = null;
  }
</script>

<Loading {loading} />

<SplitComponent>
  <div slot="sidebar">
    <button class="btn btn-secondary" on:click={clear}>
      Load another area
    </button>

    <OsmProvenance {anyEdits} />

    <Edits bind:anyEdits />

    <ProblemControls {nodes} {ways} bind:drawProblems />

    {#if pinnedWay}
      <WayDetails {pinnedWay} {drawProblemDetails} bind:showProblemDetails />
    {/if}

    <BulkOperations />
  </div>

  <div slot="map">
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

    <ProblemLayer {drawProblems} {pinnedWay} />

    <GeoJSON data={ways}>
      <LineLayer
        id="ways"
        beforeId="Road labels"
        manageHoverState
        hoverCursor="pointer"
        eventsIfTopMost
        filter={filterWays(
          onlySeverances,
          onlyModified,
          showServiceRoads,
          showKinds,
        )}
        paint={{
          "line-width": roadLineWidth(0),
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
          {@const problems = JSON.parse(props.problems)}

          <h4>Node {props.id}</h4>
          <p>Ways: {props.way_ids}</p>
          <table class="table table-bordered">
            <tbody>
              {#each Object.entries(JSON.parse(props.tags || "{}")) as [key, value]}
                <tr>
                  <td>{key}</td>
                  <td>{value}</td>
                </tr>
              {/each}
            </tbody>
          </table>

          {#if problems.length}
            <u>Problems:</u>

            {#each problems as problem}
              <p>{problem.note}</p>
            {/each}
          {/if}
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

    <Control position="top-right">
      <CollapsibleCard>
        <div slot="header">Layers</div>
        <div slot="body">
          <Checkbox bind:checked={$debugMode}>Debug mode</Checkbox>

          <Checkbox bind:checked={showNodes}>Nodes</Checkbox>

          <Checkbox bind:checked={onlyModified}>Only modified objects</Checkbox>

          <Metrics bind:showKinds>
            <Checkbox bind:checked={onlySeverances}>
              Only show major roads
            </Checkbox>

            <Checkbox bind:checked={showServiceRoads}>
              Show service roads
            </Checkbox>
          </Metrics>
        </div>
      </CollapsibleCard>
    </Control>
  </div>
</SplitComponent>
