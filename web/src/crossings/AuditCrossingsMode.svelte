<script lang="ts">
  import { Checkbox } from "svelte-utils";
  import { GeoJSON, CircleLayer, hoverStateFilter } from "svelte-maplibre";
  import { SplitComponent } from "svelte-utils/two_column_layout";
  import { backend, mode } from "../";

  let ignoreServiceRoads = false;

  $: data = JSON.parse($backend!.auditCrossings(ignoreServiceRoads));

  let crossingNodes = JSON.parse($backend!.getNodes());
  crossingNodes.features = crossingNodes.features.filter(
    (f) => f.properties.is_crossing,
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
      />
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
