<script lang="ts">
  import { PolygonToolLayer } from "maplibre-draw-polygon";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { bbox } from "svelte-utils/map";
  import { Loading } from "svelte-utils";
  import { OverpassSelector } from "svelte-utils/overpass";
  import * as backendPkg from "../../backend/pkg";
  import { backend, refreshLoadingScreen, map } from "./";
  import type { Feature, Polygon } from "geojson";

  let loading = $state("");

  let fileInput: HTMLInputElement | undefined = $state();
  async function loadFile(e: Event) {
    try {
      loading = "Loading from file";
      await refreshLoadingScreen();
      let bytes = await fileInput!.files![0].arrayBuffer();
      $backend = new backendPkg.Speedwalk(new Uint8Array(bytes), null);
      zoomFit();
    } catch (err) {
      window.alert(`Bad input file: ${err}`);
    } finally {
      loading = "";
    }
  }

  async function gotXml(xml: string, boundary: Feature<Polygon>) {
    try {
      loading = "Processing Overpass data";
      await refreshLoadingScreen();
      let bytes = new TextEncoder().encode(xml);
      $backend = new backendPkg.Speedwalk(new Uint8Array(bytes), boundary);
      zoomFit();
    } catch (err) {
      window.alert(`Couldn't import from Overpass: ${err}`);
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
    <div>
      <label class="form-label">
        Load an osm.pbf or osm.xml file
        <input
          class="form-control"
          bind:this={fileInput}
          onchange={loadFile}
          type="file"
        />
      </label>
    </div>

    <p class="fst-italic my-3">or...</p>

    <OverpassSelector
      map={$map!}
      {gotXml}
      onloading={(msg) => (loading = msg)}
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
