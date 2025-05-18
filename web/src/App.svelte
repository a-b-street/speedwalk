<script lang="ts">
  import { PolygonToolLayer } from "maplibre-draw-polygon";
  import { onMount } from "svelte";
  import { colors, type WayProps } from "./";
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
  import { OverpassSelector } from "svelte-utils/overpass";
  import { Loading } from "svelte-utils";
  import { Layout } from "svelte-utils/two_column_layout";
  import {
    emptyGeojson,
    bbox,
    constructMatchExpression,
  } from "svelte-utils/map";
  import type { Feature, LineString, FeatureCollection } from "geojson";
  import init, { Speedwalk } from "backend";
  import Metrics from "./Metrics.svelte";
  import WayDetails from "./WayDetails.svelte";

  let model: Speedwalk | undefined;
  let loading = "";
  let map: Map | undefined;
  let ways = emptyGeojson() as FeatureCollection<LineString, WayProps>;
  let pinnedWay: Feature<LineString, WayProps> | null = null;

  onMount(async () => {
    await init();
  });

  let fileInput: HTMLInputElement;
  async function loadFile(e: Event) {
    try {
      loading = "Loading from file";
      let bytes = await fileInput.files![0].arrayBuffer();
      model = new Speedwalk(new Uint8Array(bytes));
      ways = JSON.parse(model.getWays());
      zoomFit();
    } catch (err) {
      window.alert(`Bad input file: ${err}`);
    } finally {
      loading = "";
    }
  }

  async function gotXml(e: CustomEvent<{ xml: string }>) {
    try {
      // TODO Can we avoid turning into bytes?
      let bytes = new TextEncoder().encode(e.detail.xml);
      model = new Speedwalk(new Uint8Array(bytes));
      ways = JSON.parse(model.getWays());
      zoomFit();
    } catch (err) {
      window.alert(`Couldn't import from Overpass: ${err}`);
    } finally {
      loading = "";
    }
  }

  function clear() {
    model = undefined;
    ways = emptyGeojson() as FeatureCollection<LineString, WayProps>;
    pinnedWay = null;
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

<Loading {loading} progress={100} />

<Layout>
  <div slot="left">
    <h1>Speedwalk</h1>

    {#if model}
      <button on:click={clear}>Load another area</button>
    {:else if map}
      <label>
        Load an osm.pbf or osm.xml file
        <input bind:this={fileInput} on:change={loadFile} type="file" />
      </label>

      <i>or...</i>

      <OverpassSelector
        {map}
        on:gotXml={gotXml}
        on:loading={(e) => (loading = e.detail)}
        on:error={(e) => window.alert(e.detail)}
      />
    {/if}

    {#if pinnedWay}
      <WayDetails {pinnedWay} />
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

      <PolygonToolLayer />

      <GeoJSON data={pinnedWay || emptyGeojson()}>
        <LineLayer
          id="pinned"
          beforeId="Road labels"
          paint={{
            "line-width": 12,
            "line-color": "cyan",
            "line-opacity": 0.5,
          }}
        />
      </GeoJSON>

      <GeoJSON data={ways}>
        <LineLayer
          id="ways"
          beforeId="Road labels"
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

      <Control position="top-right">
        <div style:background="white" style:width="150px">
          {#if model}
            <Metrics {model} />
          {/if}
        </div>
      </Control>
    </MapLibre>
  </div>
</Layout>
