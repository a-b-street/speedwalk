<script lang="ts">
  import { PolygonToolLayer } from "maplibre-draw-polygon";
  import { onMount } from "svelte";
  import {
    backend,
    previewSidewalk,
    colors,
    type NodeProps,
    type WayProps,
  } from "./";
  import "@picocss/pico/css/pico.jade.min.css";
  import type { Map, MapMouseEvent } from "maplibre-gl";
  import {
    GeoJSON,
    MapLibre,
    CircleLayer,
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
    Popup,
  } from "svelte-utils/map";
  import type { Feature, LineString, FeatureCollection, Point } from "geojson";
  import init, { Speedwalk } from "backend";
  import Metrics from "./Metrics.svelte";
  import WayDetails from "./WayDetails.svelte";

  let loading = "";
  let map: Map | undefined;
  let nodes = emptyGeojson() as FeatureCollection<Point, NodeProps>;
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
      $backend = new Speedwalk(new Uint8Array(bytes));
      nodes = JSON.parse($backend.getNodes());
      ways = JSON.parse($backend.getWays());
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
      $backend = new Speedwalk(new Uint8Array(bytes));
      nodes = JSON.parse($backend.getNodes());
      ways = JSON.parse($backend.getWays());
      zoomFit();
    } catch (err) {
      window.alert(`Couldn't import from Overpass: ${err}`);
    } finally {
      loading = "";
    }
  }

  function clear() {
    $backend = null;
    nodes = emptyGeojson() as FeatureCollection<Point, NodeProps>;
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

  $: if (!pinnedWay) {
    $previewSidewalk = null;
  }
</script>

<Loading {loading} progress={100} />

<Layout>
  <div slot="left">
    <h1>Speedwalk</h1>

    {#if $backend}
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

      <GeoJSON data={$previewSidewalk || emptyGeojson()}>
        <LineLayer
          id="preview-sidewalk"
          beforeId="Road labels"
          paint={{
            "line-width": 5,
            "line-color": "purple",
          }}
        />
      </GeoJSON>

      <GeoJSON data={nodes}>
        <CircleLayer
          id="nodes"
          beforeId="Road labels"
          manageHoverState
          paint={{
            "circle-radius": 7,
            "circle-color": "grey",
            "circle-opacity": hoverStateFilter(0, 0.5),
            "circle-stroke-color": "black",
            "circle-stroke-width": 1,
          }}
        >
          <Popup openOn="hover" let:props>
            <p>{JSON.stringify(props)}</p>
          </Popup>
        </CircleLayer>
      </GeoJSON>

      <Control position="top-right">
        <div style:background="white" style:width="150px">
          {#if $backend}
            <Metrics />
          {/if}
        </div>
      </Control>
    </MapLibre>
  </div>
</Layout>
