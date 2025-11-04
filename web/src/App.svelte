<script lang="ts">
  import Loader from "./Loader.svelte";
  import Auth from "./Auth.svelte";
  import favicon from "../assets/favicon.ico?url";
  import { MapLibre } from "svelte-maplibre";
  import { onMount } from "svelte";
  import { backend } from "./";
  import "bootstrap/dist/css/bootstrap.min.css";
  import type { Map } from "maplibre-gl";
  import { basemapStyles, Basemaps, StandardControls } from "svelte-utils/map";
  import {
    mapContents,
    sidebarContents,
    Layout,
  } from "svelte-utils/two_column_layout";
  import * as backendPkg from "../../backend/pkg";
  import MainMode from "./main/MainMode.svelte";

  let map: Map | undefined;
  let style = basemapStyles["Maptiler OpenStreetMap"];

  onMount(async () => {
    await backendPkg.default();
  });

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

<Layout>
  <div slot="left">
    <h1>Speedwalk</h1>

    <Auth />

    {#if map}
      <div bind:this={sidebarDiv} />
    {/if}
  </div>

  <div slot="main" style="position:relative; width: 100%; height: 100vh;">
    <MapLibre
      {style}
      hash
      bind:map
      on:error={(e) => {
        // @ts-ignore ErrorEvent isn't exported
        console.log(e.detail.error);
      }}
    >
      <StandardControls {map} />
      <!--<MapContextMenu {map} />-->
      <Basemaps bind:style choice="Maptiler OpenStreetMap" />

      {#if map}
        <div bind:this={mapDiv} />

        {#if $backend}
          <MainMode {map} />
        {:else}
          <Loader {map} />
        {/if}
      {/if}
    </MapLibre>
  </div>
</Layout>
