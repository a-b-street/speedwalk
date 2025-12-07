<script lang="ts">
  import { GeoJSON, LineLayer } from "svelte-maplibre";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { backend } from "../";
  import WaysLayer from "./WaysLayer.svelte";
  import WayDetails from "../sidewalks/WayDetails.svelte";
  import type {
    FeatureCollection,
    Geometry,
    Feature,
    LineString,
  } from "geojson";
  import { roadLineWidth, type WayProps } from "../sidewalks";
  import { emptyGeojson } from "svelte-utils/map";

  // TODO Maybe these should all be separate modes. Two of them can use editing directly.
  export let problem:
    | "missing footway=crossing"
    | "possible separate sidewalk near way without it tagged"
    | "sidewalk=separate is ambiguous about the side"
    | "sidewalk:left and sidewalk:right should each be tagged as separate or no";

  let problemWays = JSON.parse($backend!.getWays());
  problemWays.features = problemWays.features.filter((f: any) =>
    f.properties.problems.some((p: any) => p.note == problem),
  );

  let pinnedWay: Feature<LineString, WayProps> | null = null;

  let showProblemDetails = true;
  let drawProblemDetails = emptyGeojson() as FeatureCollection<
    Geometry,
    { label: string; color: string }
  >;

  // Animate the problems to call attention
  let opacity = 0.5;

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

<SplitComponent>
  <div slot="sidebar">
    {#if problem == "missing footway=crossing"}
      <h4>Fix footways that should be crossings</h4>

      <p></p>
    {:else if problem == "possible separate sidewalk near way without it tagged"}
      <h4>Fix roads that have separate sidewalks but aren't tagged</h4>

      <p></p>
    {:else if problem == "sidewalk=separate is ambiguous about the side"}
      <h4>Fix ambiguous sidewalk=separate</h4>

      <p></p>
    {:else if problem == "sidewalk:left and sidewalk:right should each be tagged as separate or no"}
      <h4>Fix inconsistent tagging on each side</h4>

      <p></p>
    {/if}

    <p>{problemWays.features.length} problems</p>

    {#if pinnedWay}
      <WayDetails {pinnedWay} {drawProblemDetails} bind:showProblemDetails />
    {/if}
  </div>

  <div slot="map">
    <GeoJSON data={problemWays}>
      <LineLayer
        maxzoom={16}
        paint={{
          "line-width": roadLineWidth(15),
          "line-color": "red",
          "line-opacity": opacity,
          "line-blur": 5,
        }}
      />

      <LineLayer
        beforeId="Road labels"
        minzoom={16}
        paint={{
          "line-width": roadLineWidth(20),
          "line-color": "red",
          "line-opacity": opacity,
          "line-blur": 5,
        }}
      />
    </GeoJSON>

    <WaysLayer bind:pinnedWay bind:drawProblemDetails />
  </div>
</SplitComponent>
