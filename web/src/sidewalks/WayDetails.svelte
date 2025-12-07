<script lang="ts">
  import type {
    Feature,
    FeatureCollection,
    Geometry,
    LineString,
  } from "geojson";
  import {
    debugMode,
    backend,
    mutationCounter,
    refreshLoadingScreen,
  } from "../";
  import { Checkbox, Loading, QualitativeLegend } from "svelte-utils";
  import { kindLabels, type WayProps } from "./";

  export let pinnedWay: Feature<LineString, WayProps>;
  export let drawProblemDetails: FeatureCollection<
    Geometry,
    { label: string; color: string }
  >;
  export let showProblemDetails: boolean;

  let loading = "";

  async function setTags(tags: Array<string[]>) {
    loading = "Setting tags";
    await refreshLoadingScreen();

    try {
      $backend!.editSetTags(BigInt(pinnedWay.properties.id), tags);
      $mutationCounter++;
    } finally {
      loading = "";
    }
  }

  let roadFixTagChoices = [
    [["sidewalk:both", "separate"]],
    [
      ["sidewalk:left", "separate"],
      ["sidewalk:right", "no"],
    ],
    [
      ["sidewalk:right", "separate"],
      ["sidewalk:left", "no"],
    ],
    [["sidewalk", "both"]],
    [["sidewalk", "left"]],
    [["sidewalk", "right"]],
    [["sidewalk", "no"]],
  ];

  let footwayFixTagChoices = [
    [["footway", "sidewalk"]],
    [["footway", "crossing"]],
  ];

  async function onKeyPress(e: KeyboardEvent) {
    let choices = [] as Array<string[]>[];
    if (pinnedWay.properties.kind.startsWith("Road")) {
      choices = roadFixTagChoices;
    } else if (
      pinnedWay.properties.kind == "Sidewalk" ||
      pinnedWay.properties.kind == "Other"
    ) {
      choices = footwayFixTagChoices;
    }

    let n = parseInt(e.key);
    if (Number.isInteger(n) && n <= choices.length) {
      await setTags(choices[n - 1]);
    }
  }
</script>

<svelte:window on:keypress={onKeyPress} />

<Loading {loading} />

<div class="card mb-5">
  <div class="card-header">
    <a
      href="https://www.openstreetmap.org/way/{pinnedWay.properties.id}"
      target="_blank"
    >
      Way {pinnedWay.properties.id}
    </a>
    (
    <a
      href="https://www.openstreetmap.org/edit?way={pinnedWay.properties.id}"
      target="_blank"
    >
      <i class="fa-solid fa-pencil"></i>
    </a>
    ) : {kindLabels[pinnedWay.properties.kind]}
  </div>

  <div class="card-body">
    {#if pinnedWay.properties.problems.length}
      <u>Problems:</u>

      {#each pinnedWay.properties.problems as problem}
        <p>{problem.note}</p>
      {/each}

      {#if drawProblemDetails.features.length}
        <Checkbox bind:checked={showProblemDetails}>
          Show problem details
        </Checkbox>
      {/if}

      <QualitativeLegend
        labelColors={Object.fromEntries(
          drawProblemDetails.features.map((f) => [
            f.properties.label,
            f.properties.color,
          ]),
        )}
        itemsPerRow={1}
      />
    {/if}

    {#if pinnedWay.properties.kind.startsWith("Road")}
      <u>Current sidewalk tags</u>
      <ul>
        {#each Object.entries(pinnedWay.properties.tags) as [key, value]}
          {#if key.startsWith("sidewalk")}
            <li>{key} = {value}</li>
          {/if}
        {/each}
      </ul>

      <u>Override to these tags</u>

      {#each roadFixTagChoices.entries() as [idx, tags]}
        <div>
          <button class="btn btn-secondary mb-1" on:click={() => setTags(tags)}>
            <kbd>{idx + 1}</kbd>
            {tags.map((pair) => `${pair[0]} = ${pair[1]}`).join(", ")}
          </button>
        </div>
      {/each}
    {:else if pinnedWay.properties.kind == "Sidewalk" || pinnedWay.properties.kind == "Other"}
      <u>Set these tags</u>

      {#each footwayFixTagChoices.entries() as [idx, tags]}
        <div>
          <button class="btn btn-secondary mb-1" on:click={() => setTags(tags)}>
            <kbd>{idx + 1}</kbd>
            {tags.map((pair) => `${pair[0]} = ${pair[1]}`).join(", ")}
          </button>
        </div>
      {/each}
    {/if}

    <table class="table table-bordered">
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

    {#if $debugMode}
      <p>Nodes: {pinnedWay.properties.node_ids.join(", ")}</p>
    {/if}
  </div>
</div>
