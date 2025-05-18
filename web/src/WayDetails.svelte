<script lang="ts">
  import type { Feature, LineString } from "geojson";
  import {
    type WayProps,
    backend,
    problems,
    quickfixes,
    previewSidewalk,
  } from "./";

  export let pinnedWay: Feature<LineString, WayProps>;

  let makeSidewalk = false;
  let makeDirection: "left" | "right" = "left";
  let projectDistance = 1.0;

  function remakeSidewalk(
    makeSidewalk: boolean,
    makeDirection: "left" | "right",
    projectDistance: number,
  ) {
    if (!makeSidewalk) {
      $previewSidewalk = null;
      return;
    }
    let direction = makeDirection == "left" ? -1 : 1;
    $previewSidewalk = JSON.parse(
      $backend!.makeSidewalk(
        BigInt(pinnedWay.properties.id),
        direction * projectDistance,
      ),
    );
  }
  $: remakeSidewalk(makeSidewalk, makeDirection, projectDistance);
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
{/if}
{#if pinnedWay.properties.problem}
  <p>{problems[pinnedWay.properties.problem]}</p>
{/if}

{#if pinnedWay.properties.kind == "bad_roadway"}
  <details bind:open={makeSidewalk}>
    <summary>Create a sidewalk</summary>

    <fieldset>
      <label>
        <input type="radio" value="left" bind:group={makeDirection} />
        Left
      </label>
      <label>
        <input type="radio" value="right" bind:group={makeDirection} />
        Right
      </label>
    </fieldset>

    <label>
      <input
        type="number"
        bind:value={projectDistance}
        min="0.1"
        max="10"
        step="0.1"
      />
      Project away
    </label>

    <button>Confirm</button>
  </details>
{/if}

<table>
  {#each Object.entries(pinnedWay.properties.tags) as [key, value]}
    <tr>
      <td>{key}</td>
      <td>{value}</td>
    </tr>
  {/each}
</table>
