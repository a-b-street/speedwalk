<script lang="ts">
  import type { Feature, LineString } from "geojson";
  import { backend, mutationCounter } from "../";
  import { type WayProps } from "./";

  export let pinnedWay: Feature<LineString, WayProps>;
</script>

<div class="card mb-5">
  <div class="card-header">
    <a
      href="https://www.openstreetmap.org/way/{pinnedWay.properties.id}"
      target="_blank"
    >
      Way {pinnedWay.properties.id}
    </a>
    : {pinnedWay.properties.kind}
  </div>
  <div class="card-body">
    {#if pinnedWay.properties.kind == "Road"}
      <div class="card mb-3">
        <div class="card-header">Set old-style sidewalk tags</div>
        <div
          class="card-body"
          style="display: flex; justify-content: space-between"
        >
          {#each ["both", "left", "right", "no"] as value}
            <button
              class="btn btn-secondary"
              on:click={() => window.alert(value)}
            >
              {value}
            </button>
          {/each}
        </div>
      </div>
    {/if}

    <table class="table">
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

    <p>Nodes: {pinnedWay.properties.node_ids.join(", ")}</p>
  </div>
</div>
