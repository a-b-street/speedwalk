<script lang="ts">
  import { Checkbox } from "svelte-utils";
  import {
    LineLayer,
    GeoJSON,
    CircleLayer,
    hoverStateFilter,
  } from "svelte-maplibre";
  import { SplitComponent } from "svelte-utils/two_column_layout";
  import { backend, mode } from "../";
  import type { Feature, FeatureCollection } from "geojson";
  import { emptyGeojson } from "svelte-utils/map";

  let ignoreServiceRoads = false;

  $: data = JSON.parse($backend!.auditCrossings(ignoreServiceRoads));

  let hovered: Feature | null = null;
  $: debugArms = hovered
    ? JSON.parse(hovered.properties!.arms)
    : emptyGeojson();

  let crossingNodes = JSON.parse($backend!.getNodes()) as FeatureCollection;
  crossingNodes.features = crossingNodes.features.filter(
    (f) => f.properties!.is_crossing,
  );
</script>

<SplitComponent>
  <div slot="sidebar">
    <button class="btn btn-secondary" on:click={() => ($mode = "main")}>
      Back to main mode
    </button>

    <p>{data.features.length.toLocaleString()} junctions to audit</p>

    <Checkbox bind:checked={ignoreServiceRoads}>Ignore service roads</Checkbox>
  </div>

  <div slot="map">
    <GeoJSON {data} generateId>
      <CircleLayer
        manageHoverState
        paint={{
          "circle-radius": 15,
          "circle-color": "black",
          "circle-opacity": hoverStateFilter(0.5, 1.0),
          "circle-stroke-color": "black",
          "circle-stroke-width": 3,
        }}
        bind:hovered
      />
    </GeoJSON>

    <GeoJSON data={debugArms}>
      <LineLayer paint={{ "line-width": 6, "line-color": "blue" }} />
    </GeoJSON>

    <GeoJSON data={crossingNodes} generateId>
      <CircleLayer
        manageHoverState
        paint={{
          "circle-radius": 7,
          "circle-color": "yellow",
          "circle-opacity": hoverStateFilter(0.3, 1.0),
          "circle-stroke-color": "black",
          "circle-stroke-width": 1,
        }}
      />
    </GeoJSON>
  </div>
</SplitComponent>
