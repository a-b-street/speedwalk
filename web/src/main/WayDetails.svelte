<script lang="ts">
  import type {
    Feature,
    FeatureCollection,
    Geometry,
    LineString,
  } from "geojson";
  import { backend, mutationCounter, refreshLoadingScreen } from "../";
  import { Loading, QualitativeLegend } from "svelte-utils";
  import { kindLabels, type WayProps } from "./";

  export let pinnedWay: Feature<LineString, WayProps>;
  export let drawProblemDetails: FeatureCollection<
    Geometry,
    { label: string; color: string }
  >;

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
</script>

<Loading {loading} />

<div class="card mb-5">
  <div class="card-header">
    <a
      href="https://www.openstreetmap.org/way/{pinnedWay.properties.id}"
      target="_blank"
    >
      Way {pinnedWay.properties.id}
    </a>
    : {kindLabels[pinnedWay.properties.kind]}
  </div>

  <div class="card-body">
    {#if pinnedWay.properties.problems.length}
      <u>Problems:</u>

      {#each pinnedWay.properties.problems as problem}
        <p>{problem.note}</p>
      {/each}

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
