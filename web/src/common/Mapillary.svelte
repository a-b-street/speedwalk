<script lang="ts">
  import "mapillary-js/dist/mapillary.css";
  import { Viewer } from "mapillary-js";
  import {
    hoverStateFilter,
    VectorTileSource,
    LineLayer,
    CircleLayer,
    GeoJSON,
    SymbolLayer,
  } from "svelte-maplibre";
  import { Checkbox } from "svelte-utils";
  import { emptyGeojson } from "svelte-utils/map";
  import type { Feature } from "geojson";
  import { untrack, onDestroy } from "svelte";

  let show = $state(false);

  let container: HTMLDivElement | undefined = $state();
  let viewer: Viewer | undefined = $state();

  let cameraPosition: Feature | undefined = $state();
  let cameraBearing = $state(0);

  onDestroy(() => {
    viewer?.remove();
  });

  // TODO Or @attach?
  $effect(() => {
    if (container) {
      untrack(() => {
        viewer = new Viewer({ accessToken, container: container! });

        viewer.on("position", async (e) => {
          let pos = await viewer!.getPosition();
          cameraPosition = {
            type: "Feature",
            properties: {},
            geometry: {
              type: "Point",
              coordinates: [pos.lng, pos.lat],
            },
          };
        });
        viewer.on("bearing", (e) => {
          cameraBearing = e.bearing;
        });
      });
    }
  });

  // Making this public means anybody could burn through our quota. Embedding in git means it's
  // easier to scrape, but in any case, anybody using the app can just see the key in network
  // requests. Ideally Mapillary would restrict the domains this key can be used from. If we really
  // cared, we could proxy all requests through a server that hides the key. But it's free tier and
  // read-only access, so really doesn't matter much.
  let accessToken = "MLY|25548189401504437|3c9576aa434c09888b5de1c28f13b1df";

  function close() {
    show = false;
    viewer?.activateCover();
    cameraPosition = undefined;
    cameraBearing = 0;
  }

  async function openImage(id: string) {
    viewer?.deactivateCover();
    viewer?.resize();
    await viewer?.moveTo(id);
  }
</script>

<Checkbox bind:checked={show}>Mapillary</Checkbox>

<VectorTileSource
  tiles={[
    `https://tiles.mapillary.com/maps/vtp/mly1_public/2/{z}/{x}/{y}?access_token=${accessToken}`,
  ]}
  maxzoom={14}
>
  <LineLayer
    sourceLayer="sequence"
    paint={{
      "line-width": 2,
      "line-color": "green",
    }}
    layout={{
      visibility: show ? "visible" : "none",
    }}
  />

  <CircleLayer
    sourceLayer="image"
    manageHoverState
    paint={{
      "circle-radius": 10,
      "circle-color": hoverStateFilter("green", "orange"),
      "circle-stroke-color": "black",
      "circle-stroke-width": 1,
    }}
    layout={{
      visibility: show ? "visible" : "none",
    }}
    hoverCursor="pointer"
    onclick={(e) => openImage(e.features[0].properties!.id)}
  />
</VectorTileSource>

<GeoJSON data={cameraPosition || emptyGeojson()}>
  <SymbolLayer
    layout={{
      "icon-image": "mapillary_arrow",
      "icon-rotate": cameraBearing,
      visibility: show ? "visible" : "none",
    }}
  />
</GeoJSON>

<div class="viewer-container" style:visibility={show ? "visible" : "hidden"}>
  <button class="btn btn-primary" onclick={close}>X</button>
  <div bind:this={container} style="width: 100%; height: 100%"></div>
</div>

<style>
  .viewer-container {
    z-index: 100;
    position: absolute;
    left: 10px;
    bottom: 120px;

    width: 500px;
    height: 300px;
  }
</style>
