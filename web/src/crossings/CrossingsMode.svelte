<script lang="ts">
  import { backend, mutationCounter } from "../";
  import type { NodeProps } from "../sidewalks";
  import { SplitComponent } from "svelte-utils/two_column_layout";
  import {
    hoverStateFilter,
    Control,
    GeoJSON,
    CircleLayer,
  } from "svelte-maplibre";
  import type { FeatureCollection, Point } from "geojson";
  import { Popup } from "svelte-utils/map";
  import type { Map } from "maplibre-gl";

  export let map: Map;

  let crossings: FeatureCollection<Point, NodeProps> = {
    type: "FeatureCollection",
    features: [],
  };

  $: updateModel($mutationCounter);
  function updateModel(mutationCounter: number) {
    let nodes = JSON.parse($backend!.getNodes());
    nodes.features = nodes.features.filter(
      (f: any) => f.properties.is_crossing,
    );
    crossings = nodes;
  }
</script>

<SplitComponent>
  <div slot="sidebar"></div>

  <div slot="map">
    <GeoJSON data={crossings}>
      <CircleLayer
        id="crossings"
        beforeId="Road labels"
        manageHoverState
        paint={{
          "circle-radius": 7,
          "circle-color": "yellow",
          "circle-opacity": hoverStateFilter(1.0, 0.5),
          "circle-stroke-color": "black",
          "circle-stroke-width": 1,
        }}
      >
        <Popup openOn="hover" let:props>
          <h4>Node {props.id}</h4>
          <table>
            {#each Object.entries(JSON.parse(props.tags)) as [key, value]}
              <tr>
                <td>{key}</td>
                <td>{value}</td>
              </tr>
            {/each}
          </table>
        </Popup>
      </CircleLayer>
    </GeoJSON>

    <Control position="top-right">
      <div style:background="white" style:width="200px" style:padding="8px">
        Stuff
      </div>
    </Control>
  </div>
</SplitComponent>
