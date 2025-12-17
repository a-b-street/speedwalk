<script lang="ts">
  import { roadLineWidth, colors } from "./sidewalks";
  import { FillLayer, Popup, GeoJSON, hoverStateFilter, LineLayer } from "svelte-maplibre";
  import {
    downloadGeneratedFile,
    notNull,
    QualitativeLegend,
  } from "svelte-utils";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { constructMatchExpression, emptyGeojson } from "svelte-utils/map";
  import { backend, networkFilter } from "./";
  import CollapsibleCard from "./common/CollapsibleCard.svelte";
  import NetworkFilter from "./common/NetworkFilter.svelte";
  import SharedSidebarFooter from "./common/SharedSidebarFooter.svelte";

  $: gj = $backend
    ? JSON.parse($backend.bundleFaces())
    : emptyGeojson();
</script>

<SplitComponent>
  <div slot="sidebar">
    <h4>Bundle faces (tmp)</h4>
    <SharedSidebarFooter />
  </div>

  <div slot="map">
    <GeoJSON data={gj} generateId>
      <FillLayer manageHoverState
                 paint={{
                 "fill-color": "red",
                 "fill-opacity": hoverStateFilter(0.5, 0.8),
                 }}
                 />

      <LineLayer
        paint={{
          "line-width": 2,
          "line-color": 'black',
        }}
      />
    </GeoJSON>
  </div>
</SplitComponent>
