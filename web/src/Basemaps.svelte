<script lang="ts">
  import booleanIntersects from "@turf/boolean-intersects";
  import { backend } from "./";
  import { onMount } from "svelte";
  import gjUrl from "../assets/imagery.geojson.gz?url";
  import type { FeatureCollection, Polygon, MultiPolygon } from "geojson";
  import { Speedwalk } from "backend";

  let expandPanel = false;

  interface Props {
    name: string;
    url: string;
  }

  let gj: FeatureCollection<Polygon | MultiPolygon, Props> = {
    type: "FeatureCollection" as const,
    features: [],
  };
  onMount(async () => {
    let resp = await fetch(gjUrl);
    let data = await resp.json();
    // Some of the entries are buggy
    data.features = data.features.filter((f: any) => f.geometry != null);
    gj = data;
  });

  $: choices = getChoices(gj, $backend);
  function getChoices(
    gj: FeatureCollection<Polygon | MultiPolygon, Props>,
    backend: Speedwalk | null,
  ): Props[] {
    let result = [];
    if (backend) {
      let testPt = JSON.parse(backend.getNodes()).features[0];
      for (let f of gj.features) {
        if (booleanIntersects(f, testPt)) {
          result.push(f.properties);
        }
      }
    }
    return result;
  }
</script>

<div style="display: flex; background: white">
  <button
    type="button"
    class="outline"
    on:click={() => (expandPanel = !expandPanel)}
  >
    <i class="fa-solid fa-layer-group"></i>
  </button>

  {#if expandPanel}
    <select>
      {#each choices as props}
        <option>{props.name}</option>
      {/each}
    </select>
  {/if}
</div>
