<script lang="ts">
  import {
    GeoJSON,
    hoverStateFilter,
    CircleLayer,
    Popup,
  } from "svelte-maplibre";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { backend } from "../";
  import WaysLayer from "./WaysLayer.svelte";
  import Edits from "../sidewalks/Edits.svelte";

  // TODO Maybe these should be two separate modes -- popups are only useful in one. There's not
  // much shared.
  export let problem:
    | "missing crossing node"
    | "separate sidewalks should be continued here";

  let gj = JSON.parse($backend!.getNodes());
  gj.features = gj.features.filter((f: any) =>
    f.properties.problems.some((p: any) => p.note == problem),
  );

  // TODO This should be global state
  let anyEdits = false;
</script>

<SplitComponent>
  <div slot="sidebar">
    {#if problem == "missing crossing node"}
      <h4>Fix missing crossing nodes</h4>

      <p>
        Here are nodes on crossing ways that aren't tagged as a crossing. You
        should fix by... (TODO -- write these instructions up more clearly)
      </p>
    {:else if problem == "separate sidewalks should be continued here"}
      <h4>Fix separate sidewalks that end</h4>

      <p>
        Here are places where part of a road has separate sidewalks tagged, but
        then the next part of the road doesn't. It's best to be consistent about
        separate sidewalks along the entire length of a road. To fix:
      </p>
      <ol>
        <li>
          Go map separate sidewalks along the entire road (in another editor)
        </li>
        <li>
          Update the road tagging to indicate those separate sidewalks (here or
          in another editor)
        </li>
        <li>Refresh the data here to verify</li>
      </ol>
    {/if}

    <p>{gj.features.length} problems</p>

    <Edits bind:anyEdits />
  </div>

  <div slot="map">
    <WaysLayer />

    <GeoJSON data={gj} generateId>
      <CircleLayer
        manageHoverState
        paint={{
          "circle-radius": 10,
          "circle-color": "red",
          "circle-opacity": 0.5,
          "circle-stroke-color": "black",
          "circle-stroke-width": hoverStateFilter(1, 5),
        }}
      >
        {#if problem == "missing crossing node"}
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
        {/if}
      </CircleLayer>
    </GeoJSON>
  </div>
</SplitComponent>
