<script lang="ts">
  import { backend, prettyPrintDistance, sum } from "../";
  import { colors } from "./";

  interface Metrics {
    total_length_meters: {
      sidewalk: number;
      good_roadway: number;
      quickfix_roadway: number;
      old_style_roadway: number;
      bad_roadway: number;
      other: number;
    };
  }

  let metrics: Metrics = JSON.parse($backend!.getMetrics());

  let total = sum(Object.values(metrics.total_length_meters));

  function castKey(key: string): keyof typeof colors {
    return key as keyof typeof colors;
  }
</script>

{#each Object.entries(metrics.total_length_meters) as [key, length]}
  <div class="row">
    <div
      style:position="absolute"
      style:width={`${(100 * length) / total}%`}
      style:height="100%"
      style:background-color={colors[castKey(key)]}
    />
    <div style:position="relative">{key}: {prettyPrintDistance(length)}</div>
  </div>
{/each}

<style>
  .row {
    width: 100%;
    border: 1px solid black;
    position: relative;
  }
</style>
