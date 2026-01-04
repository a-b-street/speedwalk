<script lang="ts">
  import type { Feature, Polygon } from "geojson";
  import type { LngLat, Map } from "maplibre-gl";
  import { PolygonTool, PolygonControls } from "maplibre-draw-polygon";
  // TODO Change these back
  import { overpassQueryForPolygon } from "svelte-utils/overpass";
  import { downloadGeneratedFile, Checkbox } from "svelte-utils";

  // The caller must put PolygonToolLayer in the map
  interface Props {
    map: Map | undefined;
    onloading?: (msg: string) => void;
    onload: (osmInput: Uint8Array, boundary: Feature<Polygon> | null) => void;
    onerror?: (msg: string) => void;
  }
  let { map, onloading, onload, onerror }: Props = $props();

  let polygonTool: PolygonTool | null = $state(null);
  let saveCopy = $state(false);
  let fileInput: HTMLInputElement | undefined = $state();

  async function loadFile(e: Event) {
    try {
      onloading?.("Loading from file");
      let bytes = await fileInput!.files![0].arrayBuffer();
      onload(new Uint8Array(bytes), null);
    } catch (err) {
      onerror?.(`Bad input file: ${err}`);
    }
  }

  async function importPolygon(boundaryGj: Feature<Polygon>) {
    try {
      onloading?.("Loading from Overpass");
      let resp = await fetch(overpassQueryForPolygon(boundaryGj));
      if (!resp.ok) {
        throw new Error(
          `Overpass response is not OK: ${resp.status}. The API is often overloaded; try again in a few seconds.`,
        );
      }
      let osmXml = await resp.bytes();

      if (saveCopy) {
        let text = new TextDecoder().decode(osmXml);
        downloadGeneratedFile("osm.xml", text);
      }

      onload(new Uint8Array(osmXml), boundaryGj);
    } catch (err: any) {
      onerror?.(err.toString());
    }
  }

  function latLngToGeojson(pt: LngLat): [number, number] {
    return [pt.lng, pt.lat];
  }

  // Turn the current viewport into a rectangular boundary
  function mapBoundsToGeojson(): Feature<Polygon> {
    let b = map!.getBounds();
    return {
      type: "Feature",
      properties: {},
      geometry: {
        coordinates: [
          [
            latLngToGeojson(b.getSouthWest()),
            latLngToGeojson(b.getNorthWest()),
            latLngToGeojson(b.getNorthEast()),
            latLngToGeojson(b.getSouthEast()),
            latLngToGeojson(b.getSouthWest()),
          ],
        ],
        type: "Polygon",
      },
    };
  }

  async function importCurrentView() {
    if (!map) {
      return;
    }
    if (map.getZoom() < 13) {
      onerror?.("Zoom in more to import");
      return;
    }
    await importPolygon(mapBoundsToGeojson());
  }

  function startPolygonTool() {
    if (!map) {
      return;
    }
    polygonTool = new PolygonTool(map);
    polygonTool.startNew();
    polygonTool.addEventListenerSuccess(async (f) => {
      polygonTool = null;
      await importPolygon(f);
    });
    polygonTool.addEventListenerFailure(() => {
      polygonTool = null;
    });
  }
</script>

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

{#if polygonTool}
  <PolygonControls {polygonTool} />
{:else}
  <button class="btn btn-primary" type="button" onclick={importCurrentView}>
    Import current view
  </button>

  <p class="fst-italic my-3">or...</p>

  <button class="btn btn-primary" type="button" onclick={startPolygonTool}>
    Draw an area to import on the map
  </button>
{/if}

<Checkbox bind:checked={saveCopy}>
  Save a copy of the osm.xml after importing
</Checkbox>
