<script lang="ts">
  import { QualitativeLegend } from "svelte-utils";
  import { backend, mutationCounter, prettyPrintDistance, sum } from "../";
  import { colors } from "./";

  let roads = [
    ["RoadWithSeparate", "With separate sidewalks"],
    ["RoadWithTags", "With tagged sidewalks"],
    ["RoadWithoutSidewalksExplicit", "Tagged as no sidewalks"],
    ["RoadWithoutSidewalksImplicit", "Assumed as no sidewalks"],
    ["RoadUnknown", "Totally unknown"],
  ];

  interface Metrics {
    total_length_meters: Record<keyof typeof colors, number>;
  }

  let metrics: Metrics = JSON.parse($backend!.getMetrics());
  $: if ($mutationCounter && $backend) {
    metrics = JSON.parse($backend.getMetrics());
  }

  $: total = sum(
    roads.map(([x, _]) => metrics.total_length_meters[castKey(x)]),
  );

  $: otherColors = Object.fromEntries([
    [
      `Sidewalks: ${prettyPrintDistance(metrics.total_length_meters.Sidewalk)}`,
      colors.Sidewalk,
    ],
    [
      `Crossings: ${prettyPrintDistance(metrics.total_length_meters.Crossing)}`,
      colors.Crossing,
    ],
    [
      `Other: ${prettyPrintDistance(metrics.total_length_meters.Other)}`,
      colors.Other,
    ],
  ]);

  function castKey(key: string): keyof typeof colors {
    return key as keyof typeof colors;
  }
</script>

<div class="card mb-3">
  <div class="card-header">Roads</div>
  <div class="card-body">
    <div class="row">
      {#each roads as [key, _]}
        {@const length = metrics.total_length_meters[castKey(key)]}
        <span
          style:width={`${(100 * length) / total}%`}
          style:height="100%"
          style:background-color={colors[castKey(key)]}
        ></span>
      {/each}
    </div>

    <QualitativeLegend
      labelColors={Object.fromEntries(
        roads.map(([key, label]) => [
          `${label}: ${prettyPrintDistance(metrics.total_length_meters[castKey(key)])}`,
          colors[castKey(key)],
        ]),
      )}
      itemsPerRow={1}
    />
  </div>
</div>

<QualitativeLegend labelColors={otherColors} itemsPerRow={1} />

<style>
  .row {
    display: flex;
    flex-wrap: nowrap;
    width: 100%;
    height: 30px;
  }

  .row span {
    flex-shrink: 1;
    min-width: 0;
  }
</style>
