<script lang="ts">
  import {
    GeoJSON,
    hoverStateFilter,
    LineLayer,
    MapEvents,
    type LayerClickInfo,
  } from "svelte-maplibre";
  import type { MapMouseEvent, ExpressionSpecification } from "maplibre-gl";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { constructMatchExpression } from "svelte-utils/map";
  import { backend, map } from "./";

  let gj = JSON.parse($backend!.findConnectedComponents());

  let colors = ["#1b9e77", "#d95f02", "#7570b3", "#e7298a", "#66a61e"];
  let colorByComponent = constructMatchExpression(
    ["to-string", ["%", ["get", "component"], colors.length]],
    Object.fromEntries(colors.map((color, i) => [i.toString(), color])),
    "black",
  ) as ExpressionSpecification;

  let showComponent: number | null = null;

  function lineColor(showComponent: number | null): ExpressionSpecification {
    if (showComponent == null) {
      return colorByComponent;
    }
    return [
      "case",
      ["==", ["get", "component"], showComponent],
      colorByComponent,
      "black",
    ];
  }

  function onClickLine(e: CustomEvent<LayerClickInfo>) {
    showComponent = e.detail.features[0].properties!.component;
  }

  function onMapClick(e: CustomEvent<MapMouseEvent>) {
    // If we click off a line, clear things
    if (
      $map!.queryRenderedFeatures(e.detail.point, {
        layers: ["disconnections"],
      }).length == 0
    ) {
      showComponent = null;
    }
  }
</script>

<SplitComponent>
  <div slot="sidebar">
    <h4>Network disconnections</h4>

    <p>This shows where the separate sidewalk network is disconnected.</p>

    <p>Component sizes:</p>
    <ul>
      {#each gj.components as size, idx}
        <li style:color={colors[idx % colors.length]}>{size}</li>
      {/each}
    </ul>
  </div>

  <div slot="map">
    <MapEvents on:click={onMapClick} />

    <GeoJSON data={gj} generateId>
      <LineLayer
        id="disconnections"
        paint={{
          "line-width": hoverStateFilter(5, 10),
          "line-color": lineColor(showComponent),
        }}
        manageHoverState
        hoverCursor="pointer"
        on:click={onClickLine}
      />
    </GeoJSON>
  </div>
</SplitComponent>
