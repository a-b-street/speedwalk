<script lang="ts">
  import type { Feature, LineString } from "geojson";
  import { type WayProps, problems, quickfixes } from "./";

  export let pinnedWay: Feature<LineString, WayProps>;
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

<table style:width="100%">
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
