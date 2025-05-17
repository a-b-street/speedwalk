<script lang="ts">
  import { onMount } from "svelte";
  import "@picocss/pico/css/pico.jade.min.css";
  import type { Map, MapMouseEvent } from "maplibre-gl";
  import {
    GeoJSON,
    MapLibre,
    LineLayer,
    hoverStateFilter,
    MapEvents,
    Control,
  } from "svelte-maplibre";
  import { Layout } from "svelte-utils/two_column_layout";
  import { QualitativeLegend } from "svelte-utils";
  import {
    emptyGeojson,
    bbox,
    constructMatchExpression,
  } from "svelte-utils/map";
  import type { Feature, LineString, FeatureCollection } from "geojson";
  import init, { Speedwalk } from "backend";

  interface WayProps {
    id: number;
    tags: Record<string, string>;
    kind:
      | "sidewalk"
      | "good_roadway"
      | "quickfix_roadway"
      | "bad_roadway"
      | "other";
    fix?: string;
    problem?: string;
  }

  let model: Speedwalk | undefined;
  let map: Map | undefined;
  let ways = emptyGeojson() as FeatureCollection<LineString, WayProps>;
  let pinnedWay: Feature<LineString, WayProps> | null = null;

  let colors = {
    sidewalk: "black",
    good_roadway: "green",
    quickfix_roadway: "pink",
    bad_roadway: "red",
    other: "grey",
  };

  onMount(async () => {
    await init();
  });

  let fileInput: HTMLInputElement;
  async function loadFile(e: Event) {
    try {
      let bytes = await fileInput.files![0].arrayBuffer();
      model = new Speedwalk(new Uint8Array(bytes));
      ways = JSON.parse(model.getWays());
      zoomFit();
    } catch (err) {
      window.alert(`Bad input file: ${err}`);
    }
  }

  function zoomFit() {
    map?.fitBounds(bbox(ways), {
      animate: false,
      padding: 10,
    });
  }

  function onMapClick(e: CustomEvent<MapMouseEvent>) {
    pinnedWay = null;
    for (let rendered of map!.queryRenderedFeatures(e.detail.point, {
      layers: ["ways"],
    })) {
      // Find the original feature in the GJ, to avoid having to parse nested properties
      pinnedWay = ways.features.find((f) => f.id == rendered.id)!;
      break;
    }
  }
</script>

<Layout>
  <div slot="left">
    <h1>Speedwalk</h1>

    <label>
      Load an osm.pbf or osm.xml file
      <input bind:this={fileInput} on:change={loadFile} type="file" />
    </label>

    {#if pinnedWay}
      <div>
        <a
          href="https://www.openstreetmap.org/way/{pinnedWay.properties.id}"
          target="_blank"
        >
          Way {pinnedWay.properties.id}
        </a>
        : {pinnedWay.properties.kind}
      </div>
      {#if pinnedWay.properties.fix}
        <p>{pinnedWay.properties.fix}</p>
      {/if}
      {#if pinnedWay.properties.problem}
        <p>{pinnedWay.properties.problem}</p>
      {/if}

      <table style:width="100%">
        <thead>
          <tr>
            <th>Key</th>
            <th>Value</th>
          </tr>
        </thead>
        <tbody>
          {#each Object.entries(pinnedWay.properties.tags) as [key, value]}
            <tr>
              <td>{key}</td>
              <td>{value}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>

  <div slot="main" style="position:relative; width: 100%; height: 100vh;">
    <MapLibre
      style="https://api.maptiler.com/maps/dataviz/style.json?key=MZEJTanw3WpxRvt7qDfo"
      standardControls
      bind:map
      on:error={(e) => {
        // @ts-ignore ErrorEvent isn't exported
        console.log(e.detail.error);
      }}
    >
      <MapEvents on:click={onMapClick} />

      <GeoJSON data={ways}>
        <LineLayer
          id="ways"
          manageHoverState
          paint={{
            "line-width": hoverStateFilter(5, 8),
            "line-color": constructMatchExpression(
              ["get", "kind"],
              colors,
              "cyan",
            ),
          }}
        />
      </GeoJSON>

      <GeoJSON data={pinnedWay || emptyGeojson()}>
        <LineLayer
          id="pinned"
          paint={{ "line-width": 8, "line-color": "red", "line-opacity": 0.5 }}
        />
      </GeoJSON>

      <Control position="top-right">
        <div style:background="white" style:width="150px">
          <QualitativeLegend labelColors={colors} itemsPerRow={1} />
        </div>
      </Control>
    </MapLibre>
  </div>
</Layout>
