<script lang="ts">
  import { Checkbox, QualitativeLegend } from "svelte-utils";
  import {
    LineLayer,
    GeoJSON,
    CircleLayer,
    hoverStateFilter,
    Popup,
  } from "svelte-maplibre";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { backend } from "./";
  import type { Feature, FeatureCollection } from "geojson";
  import { emptyGeojson } from "svelte-utils/map";
  import SharedSidebarFooter from "./common/SharedSidebarFooter.svelte";

  let options = {
    only_major_roads: true,
    ignore_utility_roads: true,
    ignore_cycleways: true,
    ignore_footways: true,
    ignore_roundabouts: true,
    max_distance: 30,
  };

  $: data =
    $backend && options.max_distance
      ? (JSON.parse($backend!.auditCrossings(options)) as FeatureCollection)
      : emptyGeojson();
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
  $: debugExplicitNonCrossings = hovered
    ? JSON.parse(hovered.properties!.explicit_non_crossings)
    : emptyGeojson();
  $: crossingCount = debugCrossings.features.length;
  $: explicitNonCrossingCount = debugExplicitNonCrossings.features.length;
  $: numberIgnoredArms = hovered ? hovered.properties!.number_ignored_arms : 0;

  let crossingNodes = JSON.parse($backend!.getNodes()) as FeatureCollection;
  crossingNodes.features = crossingNodes.features.filter(
    (f) => f.properties!.is_crossing || f.properties!.is_explicit_crossing_no,
  );

  let colors = {
    "Junction to audit": "black",
    "Fully mapped junction": "green",
    Crossing: "yellow",
    "crossing=no": "purple",
  };
</script>

<SplitComponent>
  <div slot="sidebar">
    <h4>Crossings audit (experimental)</h4>

    <p>
      {completeJunctions.toLocaleString()} / {data.features.length.toLocaleString()}
      junctions have all possible crossings mapped
    </p>

    <Checkbox bind:checked={options.only_major_roads}>
      Only junctions on major roads
    </Checkbox>
    <Checkbox bind:checked={options.ignore_utility_roads}>
      Ignore <code>service</code>
      ,
      <code>track</code>
      roads
    </Checkbox>
    <Checkbox bind:checked={options.ignore_cycleways}>
      Ignore cycleways
    </Checkbox>
    <Checkbox bind:checked={options.ignore_footways}>
      Ignore <code>footway</code>
      and
      <code>path</code>
    </Checkbox>
    <Checkbox bind:checked={options.ignore_roundabouts}>
      Don't expect crossings on roundabouts
    </Checkbox>
    <div>
      <label class="form-label">
        How far away can a crossing be? (m):
        <input
          class="form-control"
          type="number"
          min="1"
          max="100"
          bind:value={options.max_distance}
        />
      </label>
    </div>

    <div class="card card-body">
      <QualitativeLegend
        labelColors={colors}
        itemsPerRow={1}
        swatchClass="circle"
      />
    </div>

    {#if hovered}
      <p class="mt-3">
        Junction has {debugArms.features.length - numberIgnoredArms} arms
        {#if numberIgnoredArms > 0}
          (plus {numberIgnoredArms}
          {numberIgnoredArms == 1 ? "arm" : "arms"} not expected to have crossings)
        {/if},
        {crossingCount}
        {crossingCount == 1 ? "crossing" : "crossings"}
        {#if explicitNonCrossingCount > 0}
          , {explicitNonCrossingCount} explicit {explicitNonCrossingCount == 1
            ? "non-crossing"
            : "non-crossings"}
        {/if}.
      </p>
      <p class="mb-3">
        <i>
          Arms are up to {options.max_distance}m away from the junction along
          the same OSM way. If the way is split before a crossing, the crossing
          won't be counted. Only the first crossing is counted per arm.
        </i>
      </p>
    {/if}

    <SharedSidebarFooter />
  </div>

  <div slot="map">
    <GeoJSON {data} generateId>
      <CircleLayer
        manageHoverState
        paint={{
          "circle-radius": 15,
          "circle-color": [
            "case",
            ["get", "complete"],
            colors["Fully mapped junction"],
            colors["Junction to audit"],
          ],
          "circle-opacity": hoverStateFilter(0.5, 1.0),
          "circle-stroke-color": "black",
          "circle-stroke-width": 3,
        }}
        bind:hovered
      />
    </GeoJSON>

    <GeoJSON data={debugArms}>
      <LineLayer
        paint={{
          "line-width": 6,
          "line-color": ["case", ["get", "has_crossing"], "blue", "red"],
        }}
      />
    </GeoJSON>

    <GeoJSON data={crossingNodes} generateId>
      <CircleLayer
        manageHoverState
        paint={{
          "circle-radius": 7,
          "circle-color": [
            "case",
            ["get", "is_explicit_crossing_no"],
            colors["crossing=no"],
            colors["Crossing"],
          ],
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
          "circle-stroke-color": "black",
          "circle-stroke-width": 3,
        }}
      />
    </GeoJSON>

    <GeoJSON data={debugExplicitNonCrossings}>
      <CircleLayer
        paint={{
          "circle-radius": 10,
          "circle-opacity": 0,
          "circle-stroke-color": "purple",
          "circle-stroke-width": 3,
        }}
      />
    </GeoJSON>
  </div>
</SplitComponent>
