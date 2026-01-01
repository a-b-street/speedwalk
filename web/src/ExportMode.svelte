<script lang="ts">
  import { roadLineWidth, colors } from "./sidewalks";
  import { Popup, GeoJSON, hoverStateFilter, LineLayer } from "svelte-maplibre";
  import { downloadGeneratedFile, QualitativeLegend } from "svelte-utils";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { constructMatchExpression, emptyGeojson } from "svelte-utils/map";
  import { backend, networkFilter } from "./";
  import CollapsibleCard from "./common/CollapsibleCard.svelte";
  import NetworkFilter from "./common/NetworkFilter.svelte";
  import SharedSidebarFooter from "./common/SharedSidebarFooter.svelte";

  let gj = $derived(
    $backend
      ? JSON.parse($backend.exportNetwork($networkFilter))
      : emptyGeojson(),
  );

  function download() {
    downloadGeneratedFile("network.geojson", JSON.stringify(gj));
  }
</script>

<SplitComponent>
  {#snippet left()}
    <h4>Export network</h4>

    <p>You can export the routeable walking network as a GeoJSON file.</p>

    <NetworkFilter />

    <button class="btn btn-primary mt-3 mb-3" onclick={download}>
      Download GeoJSON
    </button>

    <CollapsibleCard open={false}>
      {#snippet header()}Details{/snippet}
      {#snippet body()}
        <p>
          The output will have a LineString for each edge in the network. An
          edge goes between exactly two graph nodes, so one OSM way usually
          becomes many edges. Each LineString will have these properties:
        </p>
        <ul>
          <li>
            <b>node1</b>
            : The OSM node ID of the beginning of the edge. This is safer to use to
            form a graph than the first coordinate.
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
      {/snippet}
    </CollapsibleCard>

    <CollapsibleCard>
      {#snippet header()}Legend{/snippet}
      {#snippet body()}
        <QualitativeLegend labelColors={colors} itemsPerRow={1} />
      {/snippet}
    </CollapsibleCard>

    <SharedSidebarFooter />
  {/snippet}

  {#snippet main()}
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
        <Popup openOn="hover">
          {#snippet children({ data })}
            <table class="table table-bordered">
              <tbody>
                {#each Object.entries(data!.properties!) as [key, value]}
                  <tr>
                    <td>{key}</td>
                    <td>{value}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {/snippet}
        </Popup>
      </LineLayer>
    </GeoJSON>
  {/snippet}
</SplitComponent>
