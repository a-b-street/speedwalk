<script lang="ts">
  import type { Feature, LineString } from "geojson";
  import { backend, mutationCounter } from "../";
  import { type WayProps } from "./";

  export let pinnedWay: Feature<LineString, WayProps>;

  function setTags(tags: Array<string[]>) {
    $backend!.editSetTags(BigInt(pinnedWay.properties.id), tags);
    $mutationCounter++;
  }
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
    {#if pinnedWay.properties.kind == "Road" || pinnedWay.properties.kind == "RoadWithSeparate"}
      <u>Current sidewalk tags</u>
      <ul>
        {#each Object.entries(pinnedWay.properties.tags) as [key, value]}
          {#if key.startsWith("sidewalk")}
            <li>{key} = {value}</li>
          {/if}
        {/each}
      </ul>

      <u>Fix these tags</u>

      <div>
        <button
          class="btn btn-secondary mb-1"
          on:click={() => setTags([["sidewalk:both", "separate"]])}
        >
          sidewalk:both = separate
        </button>
      </div>

      <div>
        <button
          class="btn btn-secondary mb-1"
          on:click={() =>
            setTags([
              ["sidewalk:left", "separate"],
              ["sidewalk:right", "no"],
            ])}
        >
          sidewalk:left = separate, sidewalk:right = no
        </button>
      </div>

      <div>
        <button
          class="btn btn-secondary mb-1"
          on:click={() =>
            setTags([
              ["sidewalk:right", "separate"],
              ["sidewalk:left", "no"],
            ])}
        >
          sidewalk:right = separate, sidewalk:left = no
        </button>
      </div>

      {#each ["both", "left", "right", "no"] as value}
        <div>
          <button
            class="btn btn-secondary mb-1"
            on:click={() => setTags([["sidewalk", value]])}
          >
            sidewalk = {value}
          </button>
        </div>
      {/each}
    {:else if pinnedWay.properties.kind == "Sidewalk" || pinnedWay.properties.kind == "Other"}
      <u>Set these tags</u>

      <div>
        <button
          class="btn btn-secondary mb-1"
          on:click={() => setTags([["footway", "sidewalk"]])}
        >
          footway = sidewalk
        </button>
      </div>

      <div>
        <button
          class="btn btn-secondary mb-1"
          on:click={() => setTags([["footway", "crossing"]])}
        >
          footway = crossing
        </button>
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
