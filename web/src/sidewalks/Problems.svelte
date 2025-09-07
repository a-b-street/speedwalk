<script lang="ts">
  import type { FeatureCollection, LineString, Point } from "geojson";
  import { backend } from "../";
  import { emptyGeojson } from "svelte-utils/map";
  import PrevNext from "./PrevNext.svelte";

  let gj = emptyGeojson() as FeatureCollection<
    LineString | Point,
    { problem: string; osm: string }
  >;
  let idx = 0;

  function refresh() {
    gj = JSON.parse($backend!.findProblems());
    idx = 0;
  }
</script>

<div class="card mb-3">
  <div class="card-header">Find OSM problems</div>
  <div class="card-body">
    <button class="btn btn-secondary" on:click={refresh}>Find problems</button>

    {#if gj.features.length}
      <PrevNext bind:idx list={gj.features} />

      <div>
        {gj.features[idx].properties.problem}:
        <a href={gj.features[idx].properties.osm} target="_blank">OSM</a>
      </div>
    {/if}
  </div>
</div>
