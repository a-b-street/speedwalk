<script lang="ts">
  import { PolygonToolLayer } from "maplibre-draw-polygon";
  import { SplitComponent } from "svelte-utils/two_column_layout";
  import { bbox } from "svelte-utils/map";
  import { Loading } from "svelte-utils";
  import type { Map } from "maplibre-gl";
  import { OverpassSelector } from "svelte-utils/overpass";
  import * as backendPkg from "../../backend/pkg";
  import { backend } from "./";

  export let map: Map;

  let loading = "";

  let fileInput: HTMLInputElement | undefined;
  async function loadFile(e: Event) {
    try {
      loading = "Loading from file";
      let bytes = await fileInput!.files![0].arrayBuffer();
      $backend = new backendPkg.Speedwalk(new Uint8Array(bytes));
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
      $backend = new backendPkg.Speedwalk(new Uint8Array(bytes));
      zoomFit();
    } catch (err) {
      window.alert(`Couldn't import from Overpass: ${err}`);
    } finally {
      loading = "";
    }
  }

  function zoomFit() {
    map.fitBounds(bbox(JSON.parse($backend!.getNodes())), {
      animate: false,
      padding: 10,
    });
  }
</script>

<Loading {loading} />

<SplitComponent>
  <div slot="sidebar">
    <div>
      <label class="form-label">
        Load an osm.pbf or osm.xml file
        <input
          class="form-control"
          bind:this={fileInput}
          on:change={loadFile}
          type="file"
        />
      </label>
    </div>

    <p class="fst-italic my-3">or...</p>

    <OverpassSelector
      {map}
      on:gotXml={gotXml}
      on:loading={(e) => (loading = e.detail)}
      on:error={(e) => window.alert(e.detail)}
    />
  </div>

  <div slot="map">
    <PolygonToolLayer />
  </div>
</SplitComponent>
