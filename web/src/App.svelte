<script lang="ts">
  import "bootstrap/dist/css/bootstrap.min.css";
  import "bootstrap/dist/js/bootstrap.min.js";
  import "@fortawesome/fontawesome-free/css/all.min.css";
  import ReportProblem from "./ReportProblem.svelte";
  import Loader from "./Loader.svelte";
  import favicon from "../assets/favicon.ico?url";
  import arrow from "../assets/arrow.png?url";
  import { MapLibre } from "svelte-maplibre";
  import { onMount } from "svelte";
  import { basemapStyles, backend, mode, map as mapStore } from "./";
  import type { Map } from "maplibre-gl";
  import { Geocoder, StandardControls } from "svelte-utils/map";
  import Basemaps from "./Basemaps.svelte";
  import {
    mapContents,
    sidebarContents,
    Layout,
  } from "svelte-utils/top_bar_layout";
  import * as backendPkg from "../../backend/pkg";
  import MainMode from "./main/MainMode.svelte";
  import AuditCrossingsMode from "./crossings/AuditCrossingsMode.svelte";
  import DisconnectionsMode from "./DisconnectionsMode.svelte";
  import StudyAreaFade from "./StudyAreaFade.svelte";
  import NavBar from "./NavBar.svelte";

  let map: Map | undefined;
  let basemap = "Maptiler OpenStreetMap";

  $: if (map) {
    mapStore.set(map);
  }

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
  <div slot="top">
    <NavBar />
  </div>

  <div class="p-3" slot="left">
    {#if map}
      <div bind:this={sidebarDiv} />
    {/if}
  </div>

  <div slot="main" style="position:relative; width: 100%; height: 100vh;">
    <MapLibre
      style={$basemapStyles[basemap]}
      hash
      bind:map
      on:error={(e) => {
        // @ts-ignore ErrorEvent isn't exported
        console.log(e.detail.error);
      }}
      images={[{ id: "arrow", url: arrow }]}
    >
      <StandardControls {map} />
      <Geocoder {map} country={undefined} apiKey="MZEJTanw3WpxRvt7qDfo" />
      <!--<MapContextMenu {map} />-->
      <Basemaps bind:basemap />

      {#if map}
        <ReportProblem />

        <div bind:this={mapDiv} />

        {#if $backend}
          <StudyAreaFade />

          {#if $mode == "main"}
            <MainMode />
          {:else if $mode == "crossings"}
            <AuditCrossingsMode />
          {:else if $mode == "disconnections"}
            <DisconnectionsMode />
          {/if}
        {:else}
          <Loader />
        {/if}
      {/if}
    </MapLibre>
  </div>
</Layout>
