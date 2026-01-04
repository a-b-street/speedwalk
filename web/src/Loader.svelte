<script lang="ts">
  import { PolygonToolLayer } from "maplibre-draw-polygon";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { bbox } from "svelte-utils/map";
  import { Loading } from "svelte-utils";
  import * as backendPkg from "../../backend/pkg";
  import OsmLoader from "./OsmLoader.svelte";
  import { backend, refreshLoadingScreen, map } from "./";
  import type { Feature, Polygon } from "geojson";

  let loading = $state("");

  async function onload(
    osmInput: Uint8Array,
    boundary: Feature<Polygon> | null,
  ) {
    try {
      loading = "Importing OSM data";
      await refreshLoadingScreen();
      $backend = new backendPkg.Speedwalk(osmInput, boundary);
      zoomFit();
    } catch (err) {
      window.alert(`Bad OSM input: ${err}`);
    } finally {
      loading = "";
    }
  }

  function zoomFit() {
    $map!.fitBounds(bbox(JSON.parse($backend!.getNodes())), {
      animate: false,
      padding: 10,
    });
  }
</script>

<Loading {loading} />

<SplitComponent>
  {#snippet left()}
    <OsmLoader
      map={$map!}
      onloading={(msg) => (loading = msg)}
      {onload}
      onerror={(msg) => {
        window.alert(msg);
        loading = "";
      }}
    />
  {/snippet}

  {#snippet main()}
    <PolygonToolLayer />
  {/snippet}
</SplitComponent>
