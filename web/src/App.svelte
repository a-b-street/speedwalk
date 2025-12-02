<script lang="ts">
  import "bootstrap/dist/css/bootstrap.min.css";
  import "bootstrap/dist/js/bootstrap.min.js";
  import "@fortawesome/fontawesome-free/css/all.min.css";
  import ReportProblem from "./ReportProblem.svelte";
  import Loader from "./Loader.svelte";
  import Auth from "./Auth.svelte";
  import favicon from "../assets/favicon.ico?url";
  import logo from "../assets/logo.svg?url";
  import arrow from "../assets/arrow.png?url";
  import { MapLibre } from "svelte-maplibre";
  import { onMount } from "svelte";
  import { backend, mode } from "./";
  import type { Map } from "maplibre-gl";
  import {
    basemapStyles,
    Basemaps,
    Geocoder,
    StandardControls,
  } from "svelte-utils/map";
  import {
    mapContents,
    sidebarContents,
    Layout,
  } from "svelte-utils/two_column_layout";
  import { Modal } from "svelte-utils";
  import * as backendPkg from "../../backend/pkg";
  import MainMode from "./main/MainMode.svelte";
  import AuditCrossingsMode from "./crossings/AuditCrossingsMode.svelte";

  let map: Map | undefined;
  let style = basemapStyles["Maptiler OpenStreetMap"];
  let show = true;

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
    <div class="d-flex align-items-center">
      <img src={logo} style="height: 30px" class="me-3" alt="A/B Street logo" />
      <h2 class="me-3">Speedwalk</h2>
      <!-- svelte-ignore a11y-invalid-attribute -->
      <a href="#" on:click={() => (show = true)}>
        <i class="fa-solid fa-circle-info"></i>
      </a>
    </div>

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
      images={[{ id: "arrow", url: arrow }]}
    >
      <StandardControls {map} />
      <Geocoder {map} country={undefined} apiKey="MZEJTanw3WpxRvt7qDfo" />
      <!--<MapContextMenu {map} />-->
      <Basemaps bind:style choice="Maptiler OpenStreetMap" />

      {#if map}
        <ReportProblem {map} />

        <div bind:this={mapDiv} />

        {#if $backend}
          {#if $mode == "main"}
            <MainMode {map} />
          {:else}
            <AuditCrossingsMode />
          {/if}
        {:else}
          <Loader {map} />
        {/if}
      {/if}
    </MapLibre>
  </div>
</Layout>

<Modal bind:show>
  <h1>Welcome to Speedwalk</h1>

  <p>
    This tool helps you quickly assess how sidewalks are mapped in OSM. You can
    find and fix common tagging problems. The tool assumes you understand the
    correct <a
      href="https://wiki.openstreetmap.org/wiki/Sidewalks"
      target="_blank"
    >
      sidewalk mapping conventions
    </a>
    . If you are unsure about some edits you make, you can download the changeset
    file and check in JOSM, rather than uploading directly.
  </p>

  <p>
    This is an <a
      href="https://github.com/a-b-street/speedwalk"
      target="_blank"
    >
      open source project
    </a>
    developed without funding by
    <a href="https://abstreet.uk" target="_blank">A/B Street Ltd</a>
    . Please file a Github issue or contact
    <a href="mailto:dustin@abstreet.uk" target="_blank">dustin@abstreet.uk</a>
    with feedback.
    <b>This is an alpha tool; there will be problems!</b>
  </p>

  <button class="btn btn-primary" on:click={() => (show = false)}>Start</button>
</Modal>
