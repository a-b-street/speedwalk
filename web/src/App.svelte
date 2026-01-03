<script lang="ts">
  import "bootstrap/dist/css/bootstrap.min.css";
  import "bootstrap/dist/js/bootstrap.min.js";
  import "@fortawesome/fontawesome-free/css/all.min.css";
  import Loader from "./Loader.svelte";
  import favicon from "../assets/favicon.ico?url";
  import arrow from "../assets/arrow.png?url";
  import { MapLibre } from "svelte-maplibre";
  import { onMount, untrack } from "svelte";
  import { backend, mode, map as mapStore } from "./";
  import type { Map } from "maplibre-gl";
  import { basemapStyles, Geocoder, StandardControls } from "svelte-utils/map";
  import ActionBar from "./common/ActionBar.svelte";
  import { leftTarget, Layout } from "svelte-utils/top_bar_layout";
  import * as backendPkg from "../../backend/pkg";
  import SidewalksMode from "./sidewalks/SidewalksMode.svelte";
  import AuditCrossingsMode from "./crossings/AuditCrossingsMode.svelte";
  import DisconnectionsMode from "./DisconnectionsMode.svelte";
  import ExportMode from "./ExportMode.svelte";
  import StudyAreaFade from "./common/StudyAreaFade.svelte";
  import NavBar from "./common/NavBar.svelte";

  onMount(async () => {
    await backendPkg.default();
  });

  let map: Map | undefined = $state();
  let loaded = $state(false);
  $effect(() => {
    if (map) {
      mapStore.set(map);
    }
  });

  let basemap = $state("Maptiler OpenStreetMap");

  // svelte-ignore state_referenced_locally
  let initialStyle = basemapStyles.get(basemap)!;
  // TODO Upstream to svelte-maplibre. It's brittle how to detect the sources and layers that belong
  // to the basemap vs other things
  $effect(() => {
    if (basemap) {
      untrack(() => {
        map?.setStyle(basemapStyles.get(basemap)!, {
          transformStyle: (previousStyle, nextStyle) => {
            if (!previousStyle) {
              return nextStyle;
            }

            let customLayers = previousStyle.layers.filter((l) =>
              [
                "circle-",
                "line-",
                "fill-",
                "heatmap-",
                "symbol-",
                "speedwalk-",
                "edit-polygon-",
              ].some((prefix) => l.id.startsWith(prefix)),
            );
            let layers = nextStyle.layers.concat(customLayers);
            let sources = nextStyle.sources;

            for (let [key, value] of Object.entries(
              previousStyle.sources || {},
            )) {
              if (key.startsWith("geojson-") || key.startsWith("heatmap")) {
                sources[key] = value;
              }
            }

            return {
              ...nextStyle,
              sources: sources,
              layers: layers,
            };
          },
        });
      });
    }
  });
</script>

<svelte:head>
  <link rel="icon" type="image/x-icon" href={favicon} />
</svelte:head>

<Layout>
  {#snippet top()}
    <NavBar />
  {/snippet}

  {#snippet left()}
    <div class="p-3">
      {#if map}
        <div bind:this={leftTarget.value}></div>
      {/if}
    </div>
  {/snippet}

  {#snippet main()}
    <div style="position:relative; width: 100%; height: 100vh;">
      <MapLibre
        style={initialStyle}
        hash
        bind:map
        bind:loaded
        onerror={(e) => {
          console.log(e.error);
        }}
        images={[{ id: "arrow", url: arrow }]}
      >
        <StandardControls {map} />
        <Geocoder {map} {loaded} />
        <!--<MapContextMenu {map} />-->
        <ActionBar bind:basemap />

        {#if map}
          {#if $backend}
            <StudyAreaFade />

            {#key $mode}
              {#if $mode.kind == "sidewalks"}
                <SidewalksMode />
              {:else if $mode.kind == "crossings"}
                <AuditCrossingsMode />
              {:else if $mode.kind == "disconnections"}
                <DisconnectionsMode />
              {:else if $mode.kind == "export"}
                <ExportMode />
              {/if}
            {/key}
          {:else}
            <Loader />
          {/if}
        {/if}
      </MapLibre>
    </div>
  {/snippet}
</Layout>
