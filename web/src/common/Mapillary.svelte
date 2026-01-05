<script lang="ts">
  import "mapillary-js/dist/mapillary.css";
  import { Viewer } from "mapillary-js";
  import {
    hoverStateFilter,
    VectorTileSource,
    LineLayer,
    CircleLayer,
    GeoJSON,
    FillLayer,
    SymbolLayer,
  } from "svelte-maplibre";
  import { emptyGeojson } from "svelte-utils/map";
  import { untrack, onDestroy } from "svelte";
  import logo from "../../assets/Mapillary_logo.svg";
  import lineArc from "@turf/line-arc";
  import { point } from "@turf/helpers";

  let show = $state(false);

  let container: HTMLDivElement | undefined = $state();
  let viewer: Viewer | undefined = $state();

  let cameraPosition: [number, number] | undefined = $state();
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
          cameraPosition = [pos.lng, pos.lat];
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

  async function openImage(id: string) {
    viewer?.resize();
    await viewer?.moveTo(id);
  }

  let showCameraPosition = $derived.by(() => {
    if (!cameraPosition) {
      return emptyGeojson();
    }
    let radius = 30;
    let arc = lineArc(
      point(cameraPosition),
      radius,
      cameraBearing - 30,
      cameraBearing + 30,
      { units: "meters" },
    );
    // Make it a polygon
    return {
      type: "Feature" as const,
      properties: {},
      geometry: {
        type: "Polygon" as const,
        coordinates: [
          [cameraPosition, ...arc.geometry.coordinates, cameraPosition],
        ],
      },
    };
  });
</script>

<button class="btn btn-secondary" onclick={() => (show = true)} disabled={show}>
  <img src={logo} alt="Mapillary" height="30px" />
</button>

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
      "line-color": ["case", ["get", "is_pano"], "blue", "green"],
    }}
    layout={{
      visibility: show ? "visible" : "none",
    }}
  />

  <SymbolLayer
    minzoom={16}
    sourceLayer="image"
    layout={{
      visibility: show ? "visible" : "none",
      "icon-image": "chevron",
      "icon-rotate": ["get", "compass_angle"],
      "icon-offset": [15, 0],
    }}
  />

  <CircleLayer
    sourceLayer="image"
    manageHoverState
    paint={{
      "circle-radius": 8,
      "circle-color": hoverStateFilter("transparent", "orange"),
      "circle-stroke-color": "black",
      "circle-stroke-width": hoverStateFilter(0, 2),
    }}
    layout={{
      visibility: show ? "visible" : "none",
    }}
    hoverCursor="pointer"
    onclick={(e) => openImage(e.features[0].properties!.id)}
  />

  <CircleLayer
    sourceLayer="image"
    paint={{
      "circle-radius": 4,
      "circle-color": ["case", ["get", "is_pano"], "blue", "green"],
      "circle-stroke-color": "black",
      "circle-stroke-width": 1,
    }}
    layout={{
      visibility: show ? "visible" : "none",
    }}
  />
</VectorTileSource>

<GeoJSON data={showCameraPosition}>
  <FillLayer
    paint={{
      "fill-color": "orange",
      "fill-opacity": 0.8,
    }}
    layout={{
      visibility: show ? "visible" : "none",
    }}
  />

  <LineLayer
    paint={{ "line-width": 2, "line-color": "black" }}
    layout={{
      visibility: show ? "visible" : "none",
    }}
  />
</GeoJSON>

<div class="viewer-container" style:visibility={show ? "visible" : "hidden"}>
  <button class="btn btn-primary" onclick={() => (show = false)}>X</button>
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
