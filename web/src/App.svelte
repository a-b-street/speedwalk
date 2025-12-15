<script lang="ts">
  import "bootstrap/dist/css/bootstrap.min.css";
  import "bootstrap/dist/js/bootstrap.min.js";
  import "@fortawesome/fontawesome-free/css/all.min.css";
  import Loader from "./Loader.svelte";
  import favicon from "../assets/favicon.ico?url";
  import arrow from "../assets/arrow.png?url";
  import { MapLibre } from "svelte-maplibre";
  import { onMount } from "svelte";
  import { basemapStyles, backend, map as mapStore } from "./";
  import type { Map } from "maplibre-gl";
  import { Geocoder, StandardControls } from "svelte-utils/map";
  import ActionBar from "./common/ActionBar.svelte";
  import {
    mapContents,
    sidebarContents,
    Layout,
  } from "svelte-utils/top_bar_layout";
  import * as backendPkg from "../../backend/pkg";
  import SidewalksMode from "./sidewalks/SidewalksMode.svelte";
  import AuditCrossingsMode from "./crossings/AuditCrossingsMode.svelte";
  import DisconnectionsMode from "./DisconnectionsMode.svelte";
  import ExportMode from "./ExportMode.svelte";
  import StudyAreaFade from "./common/StudyAreaFade.svelte";
  import NavBar from "./common/NavBar.svelte";
  import { useMapViewport } from "./mapViewport";
  import { useModeState } from "./modeState";

  let map: Map | undefined;
  let basemap = "Maptiler OpenStreetMap";

  const mapViewport = useMapViewport();
  const modeState = useModeState();

  $: if (map) {
    mapStore.set(map);
  }

  let moveEndHandler: (() => void) | null = null;
  let mapInitialized = false;

  $: currentMode = modeState.current;
  $: currentMapViewport = mapViewport.current;

  onMount(async () => {
    await backendPkg.default();
  });

  // Initialize map from URL if present
  $: if (map && currentMapViewport && !mapInitialized) {
    const { zoom, lat, lng } = currentMapViewport;
    map.jumpTo({ center: [lng, lat], zoom });
    mapInitialized = true;
  }

  // Listen to map moveend to update URL
  $: if (map) {
    if (moveEndHandler) {
      map.off("moveend", moveEndHandler);
    }
    moveEndHandler = () => {
      if (!map) return;
      const center = map.getCenter();
      mapViewport.set({
        zoom: map.getZoom(),
        lat: center.lat,
        lng: center.lng,
      });
    };
    map.on("moveend", moveEndHandler);
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
      <ActionBar bind:basemap />

      {#if map}
        <div bind:this={mapDiv} />

        {#if $backend}
          <StudyAreaFade />

          {#key currentMode}
            {#if currentMode === "sidewalks"}
              <SidewalksMode />
            {:else if currentMode === "crossings"}
              <AuditCrossingsMode />
            {:else if currentMode === "disconnections"}
              <DisconnectionsMode />
            {:else if currentMode === "export"}
              <ExportMode />
            {/if}
          {/key}
        {:else}
          <Loader />
        {/if}
      {/if}
    </MapLibre>
  </div>
</Layout>
