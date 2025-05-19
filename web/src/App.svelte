<script lang="ts">
  import "@fortawesome/fontawesome-free/css/all.min.css";
  import { MapLibre, Control } from "svelte-maplibre";
  import { PolygonToolLayer } from "maplibre-draw-polygon";
  import { onMount } from "svelte";
  import { backend, previewSidewalk } from "./";
  import "@picocss/pico/css/pico.jade.min.css";
  import type { Map } from "maplibre-gl";
  import { OverpassSelector } from "svelte-utils/overpass";
  import { Loading } from "svelte-utils";
  import {
    mapContents,
    sidebarContents,
    Layout,
  } from "svelte-utils/two_column_layout";
  import init, { Speedwalk } from "backend";
  import Main from "./Main.svelte";

  let loading = "";
  let map: Map | undefined;

  onMount(async () => {
    await init();
  });

  let fileInput: HTMLInputElement;
  async function loadFile(e: Event) {
    try {
      loading = "Loading from file";
      let bytes = await fileInput.files![0].arrayBuffer();
      $backend = new Speedwalk(new Uint8Array(bytes));
    } catch (err) {
      window.alert(`Bad input file: ${err}`);
    } finally {
      loading = "";
    }
  }

  async function gotXml(e: CustomEvent<{ xml: string }>) {
    try {
      let bytes = new TextEncoder().encode(e.detail.xml);
      $backend = new Speedwalk(new Uint8Array(bytes));
    } catch (err) {
      window.alert(`Couldn't import from Overpass: ${err}`);
    } finally {
      loading = "";
    }
  }

  function clear() {
    $backend = null;
    $previewSidewalk = null;
  }

  let basemap: "osm" | "satellite" = "osm";
  let basemaps = {
    osm: "https://api.maptiler.com/maps/openstreetmap/style.json?key=MZEJTanw3WpxRvt7qDfo",
    satellite:
      "https://api.maptiler.com/maps/hybrid/style.json?key=MZEJTanw3WpxRvt7qDfo",
  };
  function swapBasemap() {
    basemap = basemap == "osm" ? "satellite" : "osm";
  }

  let sidebarDiv: HTMLDivElement;
  let mapDiv: HTMLDivElement;
  $: if (sidebarDiv && $sidebarContents) {
    sidebarDiv.innerHTML = "";
    sidebarDiv.appendChild($sidebarContents);
  }
  $: if (mapDiv && $mapContents) {
    mapDiv.innerHTML = "";
    mapDiv.appendChild($mapContents);
  }
</script>

<Loading {loading} />

<Layout>
  <div slot="left">
    <h1>Speedwalk</h1>

    {#if $backend}
      <button on:click={clear}>Load another area</button>
    {:else if map}
      <label>
        Load an osm.pbf or osm.xml file
        <input bind:this={fileInput} on:change={loadFile} type="file" />
      </label>

      <i>or...</i>

      <OverpassSelector
        {map}
        on:gotXml={gotXml}
        on:loading={(e) => (loading = e.detail)}
        on:error={(e) => window.alert(e.detail)}
      />
    {/if}

    <div bind:this={sidebarDiv} />
  </div>

  <div slot="main" style="position:relative; width: 100%; height: 100vh;">
    <MapLibre
      style={basemaps[basemap]}
      standardControls
      hash
      bind:map
      on:error={(e) => {
        // @ts-ignore ErrorEvent isn't exported
        console.log(e.detail.error);
      }}
    >
      {#if $backend && map}
        <div bind:this={mapDiv} />

        <Main {map} />
      {:else}
        <PolygonToolLayer />
      {/if}

      <Control position="bottom-left">
        <button type="button" class="outline" on:click={swapBasemap}>
          <i class="fa-solid fa-layer-group"></i>
          Basemap
        </button>
      </Control>
    </MapLibre>
  </div>
</Layout>
