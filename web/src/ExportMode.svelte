<script lang="ts">
  import { roadLineWidth, colors } from "./sidewalks";
  import { Popup, GeoJSON, hoverStateFilter, LineLayer } from "svelte-maplibre";
  import {
    downloadGeneratedFile,
    notNull,
    QualitativeLegend,
  } from "svelte-utils";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { constructMatchExpression } from "svelte-utils/map";
  import { backend, networkFilter } from "./";
  import CollapsibleCard from "./common/CollapsibleCard.svelte";
  import NetworkFilter from "./common/NetworkFilter.svelte";
  import SharedSidebarFooter from "./common/SharedSidebarFooter.svelte";

  $: gj = JSON.parse($backend!.exportNetwork($networkFilter));

  function download() {
    downloadGeneratedFile("network.geojson", JSON.stringify(gj));
  }
</script>

<SplitComponent>
  <div slot="sidebar">
    <h4>Export network</h4>

    <p>You can export the routeable walking network as a GeoJSON file.</p>

    <NetworkFilter />

    <button class="btn btn-primary mt-3 mb-3" on:click={download}>
      Download GeoJSON
    </button>

    <CollapsibleCard open={false}>
      <div slot="header">Details</div>
      <div slot="body">
        <p>
          The output will have a LineString for each edge in the network. An
          edge goes between exactly two graph nodes, so one OSM way usually
          becomes many edges. Each LineString will have these properties:
        </p>
        <ul>
          <li>
            <b>node1</b>
            : The OSM node ID of the beginning of the edge. This is safer to use
            to form a graph than the first coordinate.
          </li>
          <li>
            <b>node2</b>
            : The OSM node ID of the end of the edge. This is safer to use to form
            a graph than the last coordinate.
          </li>
          <li>
            <b>way</b>
            : The OSM way ID where this edge comes from.
          </li>
          <li>
            <b>kind</b>
            : Speedwalk's classification of this edge
          </li>
          <li>Each tag from the OSM way will appear as a key and value</li>
        </ul>
        <p>
          Note OSM IDs will be negative if you have run bulk operations and
          generated synthetic sidewalks or crossings.
        </p>
      </div>
    </CollapsibleCard>

    <CollapsibleCard>
      <div slot="header">Legend</div>
      <div slot="body">
        <QualitativeLegend labelColors={colors} itemsPerRow={1} />
      </div>
    </CollapsibleCard>

    <SharedSidebarFooter />
  </div>

  <div slot="map">
    <GeoJSON data={gj} generateId>
      <LineLayer
        manageHoverState
        paint={{
          "line-width": roadLineWidth(0),
          "line-color": constructMatchExpression(
            ["get", "kind"],
            colors,
            "cyan",
          ),
          "line-opacity": hoverStateFilter(1.0, 0.5),
        }}
      >
        <Popup openOn="hover" let:data>
          <table class="table table-bordered">
            <tbody>
              {#each Object.entries(notNull(notNull(data).properties)) as [key, value]}
                <tr>
                  <td>{key}</td>
                  <td>{value}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </Popup>
      </LineLayer>
    </GeoJSON>
  </div>
</SplitComponent>
