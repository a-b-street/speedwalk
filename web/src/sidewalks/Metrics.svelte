<script lang="ts">
  import { Checkbox } from "svelte-utils";
  import LegendListWrapper from "../common/LegendListWrapper.svelte";
  import LegendListItem from "../common/LegendListItem.svelte";
  import { backend, mutationCounter, prettyPrintDistance, sum } from "../";
  import { colors } from "./";
  import type { Snippet } from "svelte";

  let {
    showKinds = $bindable(),
    extraControls,
  }: { showKinds: Record<string, boolean>; extraControls: Snippet } = $props();

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

  let metrics: Metrics = $derived.by(() => {
    $mutationCounter;
    return JSON.parse($backend!.getMetrics());
  });

  // TODO Bug: 0% still shows up
  let total = $derived(
    sum(roads.map(([x, _]) => metrics.total_length_meters[castKey(x)])),
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

    <LegendListWrapper>
      {#snippet children()}
        {#each roads as [key, label]}
          <LegendListItem color={colors[castKey(key)]} swatchClass="rectangle">
            <Checkbox bind:checked={showKinds[key]}>
              {label}: {prettyPrintDistance(
                metrics.total_length_meters[castKey(key)],
              )}
            </Checkbox>
          </LegendListItem>
        {/each}
      {/snippet}
    </LegendListWrapper>

    <div class="mb-3"></div>
    {@render extraControls()}
  </div>
</div>

<LegendListWrapper>
  {#snippet children()}
    {#each nonRoads as [key, label]}
      <LegendListItem color={colors[castKey(key)]} swatchClass="rectangle">
        <Checkbox bind:checked={showKinds[key]}>
          {label}: {prettyPrintDistance(
            metrics.total_length_meters[castKey(key)],
          )}
        </Checkbox>
      </LegendListItem>
    {/each}
  {/snippet}
</LegendListWrapper>

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
