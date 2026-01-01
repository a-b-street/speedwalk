<script lang="ts">
  import { Checkbox, QualitativeLegend } from "svelte-utils";
  import {
    LineLayer,
    GeoJSON,
    CircleLayer,
    hoverStateFilter,
    Popup,
    Control,
  } from "svelte-maplibre";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { backend, mutationCounter, map } from "../";
  import type { Feature, FeatureCollection } from "geojson";
  import { emptyGeojson } from "svelte-utils/map";
  import SharedSidebarFooter from "../common/SharedSidebarFooter.svelte";
  import CollapsibleCard from "../common/CollapsibleCard.svelte";
  import BulkOperations from "./BulkOperations.svelte";
  import { getMapViewport, getIdUrl } from "../common/osmEditorUrls";
  import type { MapGeoJSONFeature } from "maplibre-gl";

  let options = $state({
    only_major_roads: true,
    ignore_utility_roads: true,
    ignore_cycleways: true,
    ignore_footways: true,
    ignore_roundabouts: true,
    max_distance: 30,
  });

  let data = $derived(
    $backend && options.max_distance && $mutationCounter >= 0
      ? (JSON.parse($backend!.auditCrossings(options)) as FeatureCollection)
      : emptyGeojson(),
  );
  let completeJunctions = $derived(
    data.features.filter((f) => f.properties!.complete).length,
  );

  let hovered: (Feature & MapGeoJSONFeature) | undefined = $state();
  let debugArms = $derived(
    hovered ? JSON.parse(hovered.properties!.arms) : emptyGeojson(),
  );
  let debugCrossings = $derived(
    hovered ? JSON.parse(hovered.properties!.crossings) : emptyGeojson(),
  );
  let debugExplicitNonCrossings = $derived(
    hovered
      ? JSON.parse(hovered.properties!.explicit_non_crossings)
      : emptyGeojson(),
  );
  let crossingCount = $derived(debugCrossings.features.length);
  let explicitNonCrossingCount = $derived(
    debugExplicitNonCrossings.features.length,
  );
  let numberIgnoredArms = $derived(
    hovered ? hovered.properties!.number_ignored_arms : 0,
  );

  let crossingNodes = $derived.by(() => {
    $mutationCounter;
    if (!$backend) {
      return emptyGeojson();
    }
    let gj = JSON.parse($backend.getNodes()) as FeatureCollection;
    gj.features = gj.features.filter(
      (f) => f.properties!.is_crossing || f.properties!.is_explicit_crossing_no,
    );
    return gj;
  });

  function clickJunction() {
    let viewport = getMapViewport($map);
    if (viewport) {
      let url = getIdUrl(viewport.zoom, viewport.lat, viewport.lng);
      window.open(url, "_blank", "noopener,noreferrer");
    }
  }

  let colors = {
    "Junction to audit": "black",
    "Fully mapped junction": "green",
    Crossing: "yellow",
    "crossing=no": "purple",
    "crossing=imaginary": "cyan",
  };
</script>

<SplitComponent>
  {#snippet left()}
    <h4>Crossings audit</h4>

    <p>
      {completeJunctions.toLocaleString()} / {data.features.length.toLocaleString()}
      junctions have all possible crossings mapped
    </p>

    {#if hovered}
      <p>
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

    <hr />

    <p>
      For each junction shown, this tool looks for crossing nodes on each arm
      (road) of the junction. Please map a crossing node on each arm by clicking
      to open in iD, then refreshing data here to check. If there's no way to
      cross an arm, use <a
        href="https://wiki.openstreetmap.org/wiki/Tag:crossing=no"
        target="_blank"
      >
        crossing=no
      </a>
      to indicate a lack of a crossing. Please ignore cases where you would not expect
      any crossing to be (and report a bug to improve this tool). And note that there
      might be mid-block crossings anywhere along a road; this tool only audits junctions.
    </p>

    <BulkOperations {options} />

    <SharedSidebarFooter />
  {/snippet}

  {#snippet main()}
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
        hoverCursor="pointer"
        onclick={clickJunction}
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
            ["get", "is_imaginary_crossing"],
            colors["crossing=imaginary"],
            colors["Crossing"],
          ],
          "circle-opacity": hoverStateFilter(0.3, 1.0),
          "circle-stroke-color": "black",
          "circle-stroke-width": 1,
        }}
      >
        <Popup openOn="hover">
          {#snippet children({ data })}
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
          {/snippet}
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

    <Control position="top-right">
      <CollapsibleCard>
        {#snippet header()}Settings{/snippet}
        {#snippet body()}
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

          <div class="card card-body mt-3">
            <QualitativeLegend
              labelColors={colors}
              itemsPerRow={1}
              swatchClass="circle"
            />
          </div>
        {/snippet}
      </CollapsibleCard>
    </Control>
  {/snippet}
</SplitComponent>
