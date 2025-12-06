<script lang="ts">
  import { GeoJSON, LineLayer, Popup } from "svelte-maplibre";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { backend } from "../";
  import WaysLayer from "./WaysLayer.svelte";
  import WayDetails from "../sidewalks/WayDetails.svelte";

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

    {#if pinnedWay}
      <WayDetails {pinnedWay} {drawProblemDetails} bind:showProblemDetails />
    {/if}
  </div>

  <div slot="map">
    <WaysLayer bind:pinnedWay />

    <GeoJSON data={problemWays}>
      <LineLayer
        paint={{
          "line-width": 20,
          "line-color": "red",
          "line-opacity": 0.5,
        }}
      />
    </GeoJSON>
  </div>
</SplitComponent>
