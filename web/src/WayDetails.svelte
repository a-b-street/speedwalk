<script lang="ts">
  import type { Feature, LineString } from "geojson";
  import {
    type WayProps,
    backend,
    problems,
    quickfixes,
    previewSidewalk,
    mutationCounter,
  } from "./";

  export let pinnedWay: Feature<LineString, WayProps>;

  let makeLeft = false;
  let distanceLeft = 1.0;
  let makeRight = false;
  let distanceRight = 1.0;

  function updateSidewalkPreview(
    makeLeft: boolean,
    distanceLeft: number,
    makeRight: boolean,
    distanceRight: number,
  ) {
    if (!makeLeft && !makeRight) {
      $previewSidewalk = null;
      return;
    }
    $previewSidewalk = JSON.parse(
      $backend!.previewSidewalk(
        BigInt(pinnedWay.properties.id),
        makeLeft ? distanceLeft : 0,
        makeRight ? distanceRight : 0,
      ),
    );
  }
  $: updateSidewalkPreview(makeLeft, distanceLeft, makeRight, distanceRight);

  function makeSidewalk() {
    $backend!.editMakeSidewalk(
      BigInt(pinnedWay.properties.id),
      makeLeft ? distanceLeft : 0,
      makeRight ? distanceRight : 0,
    );
    $mutationCounter++;
    makeLeft = false;
    makeRight = false;
  }

  function applyQuickfix() {
    $backend!.editApplyQuickfix(
      BigInt(pinnedWay.properties.id),
      pinnedWay.properties.fix!,
    );
    $mutationCounter++;
  }

  function markSeparateSidewalks() {
    $backend!.editApplyQuickfix(
      BigInt(pinnedWay.properties.id),
      "OldSidewalkSeparate",
    );
    $mutationCounter++;
  }
</script>

<div>
  <a
    href="https://www.openstreetmap.org/way/{pinnedWay.properties.id}"
    target="_blank"
  >
    Way {pinnedWay.properties.id}
  </a>
  : {pinnedWay.properties.kind}
</div>
{#if pinnedWay.properties.fix}
  <p>{quickfixes[pinnedWay.properties.fix]}</p>
  <button on:click={applyQuickfix}>Apply this fix</button>
{/if}
{#if pinnedWay.properties.problem}
  <p>{problems[pinnedWay.properties.problem]}</p>
  <button on:click={markSeparateSidewalks}>
    This road already has separate sidewalks on both sides
  </button>
{/if}

{#if pinnedWay.properties.kind == "bad_roadway"}
  <div style:background="grey" style:padding="4px">
    <h3>Create a sidewalk</h3>

    <label>
      <input type="checkbox" bind:checked={makeLeft} />
      Left
      <input
        type="number"
        bind:value={distanceLeft}
        min="0.1"
        max="10"
        step="0.1"
        disabled={!makeLeft}
      />
    </label>

    <label>
      <input type="checkbox" bind:checked={makeRight} />
      Right
      <input
        type="number"
        bind:value={distanceRight}
        min="0.1"
        max="10"
        step="0.1"
        disabled={!makeRight}
      />
    </label>

    <button on:click={makeSidewalk}>Confirm</button>
  </div>
{/if}

<table>
  {#each Object.entries(pinnedWay.properties.tags) as [key, value]}
    <tr>
      <td>{key}</td>
      <td>{value}</td>
    </tr>
  {/each}
</table>
