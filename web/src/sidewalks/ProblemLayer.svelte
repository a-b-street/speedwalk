<script lang="ts">
  import { roadLineWidth } from "./";
  import { GeoJSON, LineLayer, CircleLayer } from "svelte-maplibre";
  import type { FeatureCollection, Feature } from "geojson";
  import { isLine, isPoint } from "svelte-utils/map";

  let {
    drawProblems,
    pinnedWay,
  }: { drawProblems: FeatureCollection; pinnedWay: Feature | null } = $props();

  // Animate the problems to call attention
  let opacity = $state(0.5);

  function animate(time: DOMHighResTimeStamp) {
    let duration = 2500;
    let low = 0.35;
    let high = 0.85;

    let t = (Math.sin((time / duration) * Math.PI * 2) + 1) / 2;
    opacity = low + t * (high - low);
    requestAnimationFrame(animate);
  }
  requestAnimationFrame(animate);
</script>

<GeoJSON data={drawProblems}>
  <CircleLayer
    filter={isPoint}
    paint={{
      "circle-radius": 10,
      "circle-color": "red",
      "circle-opacity": 0.5,
      "circle-stroke-color": "black",
      "circle-stroke-width": 1,
    }}
  />

  <LineLayer
    filter={isLine}
    maxzoom={16}
    paint={{
      "line-width": roadLineWidth(15),
      "line-color": "red",
      "line-opacity": opacity,
      "line-blur": 5,
    }}
    layout={{
      visibility: pinnedWay ? "none" : "visible",
    }}
  />

  <LineLayer
    filter={isLine}
    beforeId="Road labels"
    minzoom={16}
    paint={{
      "line-width": roadLineWidth(20),
      "line-color": "red",
      "line-opacity": opacity,
      "line-blur": 5,
    }}
    layout={{
      visibility: pinnedWay ? "none" : "visible",
    }}
  />
</GeoJSON>
