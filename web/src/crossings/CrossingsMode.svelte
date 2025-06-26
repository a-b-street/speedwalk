<script lang="ts">
  import { backend, mutationCounter } from "../";
  import type { NodeProps, WayProps } from "../sidewalks";
  import { SplitComponent } from "svelte-utils/two_column_layout";
  import {
    hoverStateFilter,
    Control,
    GeoJSON,
    CircleLayer,
    LineLayer,
  } from "svelte-maplibre";
  import type { LineString, FeatureCollection, Point } from "geojson";
  import { constructMatchExpression, Popup } from "svelte-utils/map";

  let onlyMainRoads = true;

  let ways: FeatureCollection<LineString, WayProps> = {
    type: "FeatureCollection",
    features: [],
  };
  let crossings: FeatureCollection<Point, NodeProps> = {
    type: "FeatureCollection",
    features: [],
  };
  let sideRoads: FeatureCollection<Point> = {
    type: "FeatureCollection",
    features: [],
  };

  $: updateModel($mutationCounter, onlyMainRoads);
  function updateModel(mutationCounter: number, onlyMainRoads: boolean) {
    ways = JSON.parse($backend!.getWays());

    let nodes = JSON.parse($backend!.getNodes());
    nodes.features = nodes.features.filter(
      (f: any) => f.properties.is_crossing,
    );
    crossings = nodes;

    sideRoads = JSON.parse($backend!.getSideRoads(onlyMainRoads));
  }

  let kindColors = {
    sidewalk: "black",
    good_roadway: "red",
    quickfix_roadway: "red",
    bad_roadway: "red",
    old_style_roadway: "purple",
    other: "grey",
  };
</script>

<SplitComponent>
  <div slot="sidebar">
    <label>
      <input type="checkbox" bind:checked={onlyMainRoads} />
      Only audit along main roads
    </label>
  </div>

  <div slot="map">
    <GeoJSON data={ways}>
      <LineLayer
        id="ways"
        beforeId="Road labels"
        manageHoverState
        eventsIfTopMost
        paint={{
          "line-width": hoverStateFilter(5, 8),
          "line-color": constructMatchExpression(
            ["get", "kind"],
            kindColors,
            "cyan",
          ),
        }}
      >
        <Popup openOn="hover" let:props>
          <h4>Way {props.id}</h4>
          <p>{props.num_crossings} crossings</p>
        </Popup>
      </LineLayer>
    </GeoJSON>

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

    <GeoJSON data={sideRoads}>
      <CircleLayer
        id="side-roads"
        beforeId="Road labels"
        paint={{
          "circle-radius": 20,
          "circle-color": "cyan",
          "circle-opacity": 0.5,
        }}
      />
    </GeoJSON>

    <Control position="top-right">
      <div style:background="white" style:width="200px" style:padding="8px">
        Stuff
      </div>
    </Control>
  </div>
</SplitComponent>
