<script lang="ts">
  import { Checkbox } from "svelte-utils";
  import {
    LineLayer,
    GeoJSON,
    CircleLayer,
    hoverStateFilter,
    Popup,
  } from "svelte-maplibre";
  import { SplitComponent } from "svelte-utils/two_column_layout";
  import { backend, mode } from "../";
  import type { Feature, FeatureCollection } from "geojson";
  import { emptyGeojson } from "svelte-utils/map";

  let ignoreServiceRoads = false;

  $: data = JSON.parse(
    $backend!.auditCrossings(ignoreServiceRoads),
  ) as FeatureCollection;
  $: completeJunctions = data.features.filter(
    (f) => f.properties!.complete,
  ).length;

  let hovered: Feature | null = null;
  $: debugArms = hovered
    ? JSON.parse(hovered.properties!.arms)
    : emptyGeojson();
  $: debugCrossings = hovered
    ? JSON.parse(hovered.properties!.crossings)
    : emptyGeojson();

  let crossingNodes = JSON.parse($backend!.getNodes()) as FeatureCollection;
  crossingNodes.features = crossingNodes.features.filter(
    (f) => f.properties!.is_crossing,
  );
</script>

<SplitComponent>
  <div slot="sidebar">
    <h4>Crossings audit (experimental)</h4>

    <button class="btn btn-secondary" on:click={() => ($mode = "main")}>
      Back to main mode
    </button>

    <p>
      {completeJunctions.toLocaleString()} / {data.features.length.toLocaleString()}
      junctions have all possible crossings mapped
    </p>

    <Checkbox bind:checked={ignoreServiceRoads}>Ignore service roads</Checkbox>

    {#if hovered}
      <p class="mt-5">
        Junction has {debugArms.features.length} arms, {debugCrossings.features
          .length} crossings
      </p>
    {/if}
  </div>

  <div slot="map">
    <GeoJSON {data} generateId>
      <CircleLayer
        manageHoverState
        paint={{
          "circle-radius": 15,
          "circle-color": ["case", ["get", "complete"], "green", "black"],
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
      >
        <Popup openOn="hover" let:data>
          {@const props = data?.properties ?? {}}
          <h4>Node {props.id}</h4>
          <table class="table table-bordered">
            <tbody>
              {#each Object.entries(JSON.parse(props.tags || "{}")) as [key, value]}
                <tr>
                  <td>{key}</td>
                  <td>{value}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </Popup>
      </CircleLayer>
    </GeoJSON>

    <GeoJSON data={debugCrossings}>
      <CircleLayer
        paint={{
          "circle-radius": 10,
          "circle-opacity": 0,
          "circle-stroke-color": "red",
          "circle-stroke-width": 3,
        }}
      />
    </GeoJSON>
  </div>
</SplitComponent>
