<script lang="ts">
  import type { Feature, LineString } from "geojson";
  import { backend, mutationCounter } from "../";
  import { type WayProps, problems, quickfixes } from "./";

  export let pinnedWay: Feature<LineString, WayProps>;

  function applyQuickfix() {
    $backend!.editApplyQuickfix(
      BigInt(pinnedWay.properties.id),
      pinnedWay.properties.fix!,
    );
    $mutationCounter++;
  }

  function doSpecificQuickfix(fix: string) {
    $backend!.editApplyQuickfix(BigInt(pinnedWay.properties.id), fix);
    $mutationCounter++;
  }

  function capitalize(word: string) {
    return word.charAt(0).toUpperCase() + word.slice(1);
  }
</script>

<h5>
  <a
    href="https://www.openstreetmap.org/way/{pinnedWay.properties.id}"
    target="_blank"
  >
    Way {pinnedWay.properties.id}
  </a>
  : {pinnedWay.properties.kind}
</h5>

{#if pinnedWay.properties.fix}
  <div class="card mb-3">
    <div class="card-header">{quickfixes[pinnedWay.properties.fix]}</div>
    <div class="card-body">
      <button class="btn btn-secondary" on:click={applyQuickfix}>
        Apply this fix
      </button>
    </div>
  </div>
{/if}

{#if pinnedWay.properties.problem}
  <div class="card mb-3">
    <div class="card-header">{problems[pinnedWay.properties.problem]}</div>
    <div class="card-body">
      <button
        class="btn btn-secondary"
        on:click={() => doSpecificQuickfix("OldSidewalkSeparate")}
      >
        This road already has separate sidewalks on both sides
      </button>
    </div>
  </div>
{/if}

{#if pinnedWay.properties.kind == "bad_roadway"}
  <div class="card mb-3">
    <div class="card-header">Set old-style sidewalk tags</div>
    <div
      class="card-body"
      style="display: flex; justify-content: space-between"
    >
      {#each ["both", "left", "right", "no"] as value}
        <button
          class="btn btn-secondary"
          on:click={() =>
            doSpecificQuickfix(`SetOldSidewalk${capitalize(value)}`)}
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
