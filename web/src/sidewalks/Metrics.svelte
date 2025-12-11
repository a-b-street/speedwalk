<script lang="ts">
  import { Checkbox } from "svelte-utils";
  import { backend, mutationCounter, prettyPrintDistance, sum } from "../";
  import { colors } from "./";

  export let showKinds: Record<string, boolean>;

  let roads = [
    ["RoadWithSeparate", "With separate sidewalks"],
    ["RoadWithTags", "With tagged sidewalks"],
    ["RoadWithoutSidewalksExplicit", "Tagged as no sidewalks"],
    ["RoadWithoutSidewalksImplicit", "Assumed as no sidewalks"],
    ["RoadUnknown", "Totally unknown"],
  ];
  let nonRoads = [
    ["Sidewalk", "Sidewalks"],
    ["Crossing", "Crossings"],
    ["Other", "Other"],
  ];

  interface Metrics {
    total_length_meters: Record<keyof typeof colors, number>;
  }

  let metrics: Metrics = JSON.parse($backend!.getMetrics());
  $: if ($mutationCounter && $backend) {
    metrics = JSON.parse($backend.getMetrics());
  }

  // TODO Bug: 0% still shows up
  $: total = sum(
    roads.map(([x, _]) => metrics.total_length_meters[castKey(x)]),
  );

  function castKey(key: string): keyof typeof colors {
    return key as keyof typeof colors;
  }
</script>

<div class="card mb-3">
  <div class="card-header">Roads</div>
  <div class="card-body">
    <div class="row mb-1">
      {#each roads as [key, _]}
        {@const length = metrics.total_length_meters[castKey(key)]}
        <span
          style:width={`${(100 * length) / total}%`}
          style:height="100%"
          style:background-color={colors[castKey(key)]}
        ></span>
      {/each}
    </div>

    {#each roads as [key, label]}
      <div style="display: flex; align-items: center; gap: 6px;">
        <Checkbox bind:checked={showKinds[key]}>
          <span
            class="color-swatch"
            style:background={colors[castKey(key)]}
          ></span>
          {label}: {prettyPrintDistance(
            metrics.total_length_meters[castKey(key)],
          )}
        </Checkbox>
      </div>
    {/each}

    <div class="mb-3"></div>
    <slot />
  </div>
</div>

{#each nonRoads as [key, label]}
  <div style="display: flex; align-items: center; gap: 6px;">
    <Checkbox bind:checked={showKinds[key]}>
      <span class="color-swatch" style:background={colors[castKey(key)]}></span>
      {label}: {prettyPrintDistance(metrics.total_length_meters[castKey(key)])}
    </Checkbox>
  </div>
{/each}

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

  .color-swatch {
    height: 16px;
    width: 24px;
    border: 1px solid #888;
    display: inline-block;
  }
</style>
