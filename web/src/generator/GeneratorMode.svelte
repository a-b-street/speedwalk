<script lang="ts">
  import Jumbotron from "../common/Jumbotron.svelte";
  import GeneratorBulkOperations from "./GeneratorBulkOperations.svelte";
  import { backend, mutationCounter, refreshLoadingScreen } from "../";
  import {
    roadLineWidth,
    colors,
    sidewalkLegendColors,
    type NodeProps,
    type WayProps,
  } from "../sidewalks";
  import { crossingLineLegendColors } from "../crossings/legend";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { GeoJSON, LineLayer, CircleLayer } from "svelte-maplibre";
  import { hoverStateFilter } from "svelte-maplibre";
  import { Loading } from "svelte-utils";
  import {
    crossingLegendItems,
    getCrossingNodeCirclePaint,
  } from "../crossings/legend";
  import LegendList from "../common/LegendList.svelte";
  import CollapsibleCard from "../common/CollapsibleCard.svelte";
  import { Control } from "svelte-maplibre";
  import type { FeatureCollection, LineString, Point } from "geojson";
  import type { ExpressionSpecification } from "maplibre-gl";

  let nodes: FeatureCollection<Point, NodeProps> = $state.raw({
    type: "FeatureCollection",
    features: [],
  });
  let ways: FeatureCollection<LineString, WayProps> = $state.raw({
    type: "FeatureCollection",
    features: [],
  });
  let loading = $state("");

  $effect(() => {
    $mutationCounter;
    if (!$backend) return;
    loading = "Recalculating model";
    refreshLoadingScreen().then(() => {
      try {
        nodes = JSON.parse($backend!.getNodes());
        ways = JSON.parse($backend!.getWays());
      } finally {
        loading = "";
      }
    });
  });

  let crossingNodes = $derived.by(() => {
    if (!nodes.features.length) return nodes;
    return {
      ...nodes,
      features: nodes.features.filter(
        (f) =>
          f.properties?.is_crossing ||
          f.properties?.is_explicit_crossing_no ||
          f.properties?.is_generated_crossing,
      ),
    };
  });

  const roadKinds = [
    "RoadWithSeparate",
    "RoadWithTags",
    "RoadWithoutSidewalksExplicit",
    "RoadWithoutSidewalksImplicit",
    "RoadUnknown",
  ];

  /** Generator-only road legend: major = dark blue, minor = light blue. */
  const generatorRoadLegendColors = {
    "Major roads": "#1565c0",
    "Minor roads": "#64b5f6",
  } as const;

  /** Line color by kind: roads by major/minor, sidewalks/crossings by OSM vs generated. */
  const generatorWayLineColor = [
    "case",
    ["in", ["get", "kind"], ["literal", roadKinds]],
    [
      "case",
      ["get", "is_severance"],
      generatorRoadLegendColors["Major roads"],
      generatorRoadLegendColors["Minor roads"],
    ],
    ["==", ["get", "kind"], "Sidewalk"],
    [
      "case",
      ["get", "modified"],
      sidewalkLegendColors["Generated sidewalks"],
      sidewalkLegendColors["Sidewalks from OSM"],
    ],
    ["==", ["get", "kind"], "Crossing"],
    [
      "case",
      ["get", "modified"],
      crossingLineLegendColors["Generated crossings"],
      crossingLineLegendColors["Crossings from OSM"],
    ],
    colors.Other,
  ] as ExpressionSpecification;
</script>

<Loading {loading} />

<SplitComponent>
  {#snippet left()}
    <Jumbotron
      title="Generator"
      lead="Add generated sidewalks and crossings to the network for analysis. Use the actions below; do not upload generated data to OSM without review."
    />

    <GeneratorBulkOperations />
  {/snippet}

  {#snippet main()}
    <GeoJSON data={ways}>
      <LineLayer
        id="speedwalk-generator-ways"
        beforeId="Road labels"
        manageHoverState
        paint={{
          "line-width": roadLineWidth(0),
          "line-color": generatorWayLineColor,
        }}
      />
    </GeoJSON>

    <GeoJSON data={crossingNodes}>
      <CircleLayer
        id="speedwalk-generator-nodes"
        beforeId="Road labels"
        manageHoverState
        paint={getCrossingNodeCirclePaint(hoverStateFilter(0.3, 1.0))}
      />
    </GeoJSON>

    <Control position="top-right">
      <CollapsibleCard>
        {#snippet header()}Legend{/snippet}
        {#snippet body()}
          <h6 class="mb-2">Roads</h6>
          <LegendList
            items={Object.entries(generatorRoadLegendColors).map(
              ([label, color]) => ({
                label,
                color,
              }),
            )}
            swatchClass="rectangle"
          />
          <h6 class="mb-2 mt-3">Sidewalks</h6>
          <LegendList
            items={Object.entries(sidewalkLegendColors).map(
              ([label, color]) => ({
                label,
                color,
              }),
            )}
            swatchClass="rectangle"
          />
          <h6 class="mb-2 mt-3">Crossings (Lines)</h6>
          <LegendList
            items={Object.entries(crossingLineLegendColors).map(
              ([label, color]) => ({ label, color }),
            )}
            swatchClass="rectangle"
          />
          <h6 class="mb-2 mt-3">Crossing</h6>
          <LegendList items={crossingLegendItems} swatchClass="circle" />
          <h6 class="mb-2 mt-3">Other</h6>
          <LegendList
            items={[{ label: "Other ways", color: colors.Other }]}
            swatchClass="rectangle"
          />
        {/snippet}
      </CollapsibleCard>
    </Control>
  {/snippet}
</SplitComponent>
