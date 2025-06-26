<script lang="ts">
  import type { Feature, LineString } from "geojson";
  import { backend, mutationCounter } from "../";
  import { type WayProps, problems, quickfixes, previewSidewalk } from "./";

  export let pinnedWay: Feature<LineString, WayProps>;

  let makeLeft = false;
  let distanceLeft = 3.0;
  let makeRight = false;
  let distanceRight = 3.0;

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
    try {
      $previewSidewalk = JSON.parse(
        $backend!.previewSidewalk(
          BigInt(pinnedWay.properties.id),
          makeLeft ? distanceLeft : 0,
          makeRight ? distanceRight : 0,
        ),
      );
    } catch (err) {
      window.alert(err);
    }
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

  function doSpecificQuickfix(fix: string) {
    $backend!.editApplyQuickfix(BigInt(pinnedWay.properties.id), fix);
    $mutationCounter++;
  }

  function capitalize(word: string) {
    return word.charAt(0).toUpperCase() + word.slice(1);
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
  <button on:click={() => doSpecificQuickfix("OldSidewalkSeparate")}>
    This road already has separate sidewalks on both sides
  </button>
{/if}

{#if pinnedWay.properties.kind == "bad_roadway"}
  <div style:background="grey" style:padding="4px">
    <h3>Set old-style sidewalk tags</h3>
    <div style="display: flex; justify-content: space-between">
      {#each ["both", "left", "right", "no"] as value}
        <button
          on:click={() =>
            doSpecificQuickfix(`SetOldSidewalk${capitalize(value)}`)}
        >
          {value}
        </button>
      {/each}
    </div>
  </div>
{/if}

{#if pinnedWay.properties.kind == "bad_roadway" || pinnedWay.properties.kind == "old_style_roadway"}
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
