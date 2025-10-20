<script lang="ts">
  import type { Map } from "maplibre-gl";
  import type { Feature, FeatureCollection, LineString, Point } from "geojson";
  import { backend } from "../";
  import { Checkbox } from "svelte-utils";
  import { bbox, emptyGeojson } from "svelte-utils/map";
  import PrevNext from "./PrevNext.svelte";

  export let map: Map;
  export let drawProblems: FeatureCollection;

  let gj = emptyGeojson() as FeatureCollection<
    LineString | Point,
    { problem: string; osm: string }
  >;
  let problemTypes = new Set<string>();

  let idx = 0;
  let showAll = true;
  let filterType = "";

  function refresh() {
    gj = JSON.parse($backend!.findProblems());
    idx = 0;

    if (filterType == "") {
      problemTypes = new Set();
      for (let f of gj.features) {
        problemTypes.add(f.properties.problem);
      }
      problemTypes = problemTypes;
    } else {
      gj.features = gj.features.filter(
        (f) => f.properties.problem == filterType,
      );
      gj = gj;
    }
  }

  $: drawProblems = gj.features.length
    ? showAll
      ? gj
      : { type: "FeatureCollection", features: [gj.features[idx]] }
    : emptyGeojson();

  $: if (gj.features.length && idx != -1 && !showAll) {
    showFeature(gj.features[idx]);
  }

  function showFeature(f: Feature) {
    if (f.geometry.type == "Point") {
      map.flyTo({
        center: f.geometry.coordinates as [number, number],
        duration: 500,
      });
    } else {
      map.fitBounds(bbox(f), {
        padding: 200,
        duration: 500,
      });
    }
  }
</script>

<div class="card mb-3">
  <div class="card-header">Find OSM problems</div>
  <div class="card-body">
    <button class="btn btn-secondary mb-3" on:click={refresh}>
      Find problems
    </button>

    {#if gj.features.length}
      <label>
        Only show problems
        <select class="form-select" bind:value={filterType} on:change={refresh}>
          <option value="">All</option>
          {#each problemTypes as x}
            <option value={x}>{x}</option>
          {/each}
        </select>
      </label>

      <Checkbox bind:checked={showAll}>Show all problems</Checkbox>

      {#if !showAll}
        <PrevNext bind:idx list={gj.features} />

        <div>
          {gj.features[idx].properties.problem}:
          <a href={gj.features[idx].properties.osm} target="_blank">OSM</a>
        </div>
      {/if}
    {/if}
  </div>
</div>
