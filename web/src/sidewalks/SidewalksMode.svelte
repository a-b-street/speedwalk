<script lang="ts">
  import WaysLayer from "./WaysLayer.svelte";
  import ProblemControls from "./ProblemControls.svelte";
  import ProblemLayer from "./ProblemLayer.svelte";
  import CollapsibleCard from "../common/CollapsibleCard.svelte";
  import Edits from "./Edits.svelte";
  import SharedSidebarFooter from "../common/SharedSidebarFooter.svelte";
  import BulkOperations from "./BulkOperations.svelte";
  import {
    backend,
    mutationCounter,
    refreshLoadingScreen,
    debugMode,
  } from "../";
  import { colors, type NodeProps, type WayProps } from "./";
  import type { ExpressionSpecification } from "maplibre-gl";
  import { GeoJSON, CircleLayer, Control, Popup } from "svelte-maplibre";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { emptyGeojson } from "svelte-utils/map";
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
  let showRoadSides = false;

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

    if (pinnedWay) {
      let findId = pinnedWay.id;
      pinnedWay = ways.features.find((f) => f.id == findId)!;
    }
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
</script>

<Loading {loading} />

<SplitComponent>
  {#snippet left()}
    <Edits />

    <ProblemControls {nodes} {ways} bind:drawProblems />

    {#if pinnedWay}
      <WayDetails {pinnedWay} {drawProblemDetails} bind:showProblemDetails />
    {/if}

    <BulkOperations />

    <SharedSidebarFooter />
  {/snippet}

  {#snippet main()}
    <WaysLayer
      bind:pinnedWay
      bind:drawProblemDetails
      {showProblemDetails}
      {nodes}
      {ways}
      {showRoadSides}
      filterWays={filterWays(
        onlySeverances,
        onlyModified,
        showServiceRoads,
        showKinds,
      )}
    />

    <ProblemLayer {drawProblems} {pinnedWay} />

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

            <Checkbox bind:checked={showRoadSides}>
              Show each side with sidewalks ✅ or not ❌
            </Checkbox>
          </Metrics>
        </div>
      </CollapsibleCard>
    </Control>
  {/snippet}
</SplitComponent>
