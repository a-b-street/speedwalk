<script lang="ts">
  import Jumbotron from "../common/Jumbotron.svelte";
  import GeneratorBulkOperations from "./GeneratorBulkOperations.svelte";
  import { backend, mutationCounter, refreshLoadingScreen } from "../";
  import { roadLineWidth } from "../sidewalks";
  import type { NodeProps, WayProps } from "../sidewalks";
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
          "line-color": ["case", ["get", "modified"], "blue", "black"],
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
          <h6 class="mb-2">Crossing</h6>
          <LegendList items={crossingLegendItems} swatchClass="circle" />
        {/snippet}
      </CollapsibleCard>
    </Control>
  {/snippet}
</SplitComponent>
