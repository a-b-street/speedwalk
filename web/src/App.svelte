<script lang="ts">
  import favicon from "../assets/favicon.ico?url";
  import "@fortawesome/fontawesome-free/css/all.min.css";
  import { MapLibre, Control } from "svelte-maplibre";
  import { PolygonToolLayer } from "maplibre-draw-polygon";
  import { onMount } from "svelte";
  import { backend, mode } from "./";
  import { previewSidewalk } from "./sidewalks/";
  import "@picocss/pico/css/pico.jade.min.css";
  import type { Map } from "maplibre-gl";
  import { bbox } from "svelte-utils/map";
  import { OverpassSelector } from "svelte-utils/overpass";
  import { Loading } from "svelte-utils";
  import {
    mapContents,
    sidebarContents,
    Layout,
  } from "svelte-utils/two_column_layout";
  import init, { Speedwalk } from "backend";
  import SidewalksMode from "./sidewalks/SidewalksMode.svelte";
  import CrossingsMode from "./crossings/CrossingsMode.svelte";
  import Edits from "./Edits.svelte";
  import Basemaps from "./Basemaps.svelte";

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
      zoomFit();
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
      zoomFit();
    } catch (err) {
      window.alert(`Couldn't import from Overpass: ${err}`);
    } finally {
      loading = "";
    }
  }

  function zoomFit() {
    map!.fitBounds(bbox(JSON.parse($backend!.getNodes())), {
      animate: false,
      padding: 10,
    });
  }

  function clear() {
    $backend = null;
    $previewSidewalk = null;
  }

  let basemap =
    "https://api.maptiler.com/maps/openstreetmap/style.json?key=MZEJTanw3WpxRvt7qDfo";

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

<svelte:head>
  <link rel="icon" type="image/x-icon" href={favicon} />
</svelte:head>

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

    {#if $backend}
      <Edits />
      <div>
        <button
          on:click={() => ($mode = "sidewalks")}
          disabled={$mode == "sidewalks"}
        >
          Sidewalks
        </button>
        <button
          on:click={() => ($mode = "crossings")}
          disabled={$mode == "crossings"}
        >
          Crossings
        </button>
      </div>
    {/if}

    <div bind:this={sidebarDiv} />
  </div>

  <div slot="main" style="position:relative; width: 100%; height: 100vh;">
    <MapLibre
      style={basemap}
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

        {#if $mode == "sidewalks"}
          <SidewalksMode {map} />
        {:else if $mode == "crossings"}
          <CrossingsMode />
        {/if}
      {:else}
        <PolygonToolLayer />
      {/if}

      <Control position="bottom-left">
        <Basemaps />
      </Control>
    </MapLibre>
  </div>
</Layout>
