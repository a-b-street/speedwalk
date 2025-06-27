<script lang="ts">
  import type { StyleSpecification } from "maplibre-gl";
  import booleanIntersects from "@turf/boolean-intersects";
  import { backend } from "./";
  import { onMount } from "svelte";
  import gjUrl from "../assets/imagery.geojson.gz?url";
  import type { FeatureCollection, Polygon, MultiPolygon } from "geojson";
  import { Speedwalk } from "backend";

  export let style: StyleSpecification | string;
  let expandPanel = false;
  let currentBasemap = "";

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

  $: updateStyle(currentBasemap);
  function updateStyle(currentBasemap: string) {
    if (currentBasemap == "") {
      style =
        "https://api.maptiler.com/maps/openstreetmap/style.json?key=MZEJTanw3WpxRvt7qDfo";
      return;
    }

    let props = choices.find((props) => props.name == currentBasemap)!;

    style = {
      version: 8,
      sources: {
        "raster-tiles": {
          type: "raster",
          tiles: [props.url],
          tileSize: 256,
          attribution: props.attribution?.text || "",
        },
      },
      layers: [
        {
          id: "raster-basemap",
          type: "raster",
          source: "raster-tiles",
        },
      ],
    };
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
    <select bind:value={currentBasemap}>
      <option value="">Maptiler OSM</option>
      {#each choices as props}
        <option value={props.name}>{props.name}</option>
      {/each}
    </select>
  {/if}
</div>
