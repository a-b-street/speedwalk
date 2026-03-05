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
  import { Loading } from "svelte-utils";
  import { kindLabels, type WayProps } from "./";
  import Problems from "./way-details/Problems.svelte";
  import CenterlineTagActions from "./way-details/CenterlineTagActions.svelte";
  import SidepathTagActions from "./way-details/SidepathTagActions.svelte";
  import CurrentTagsTable from "./way-details/CurrentTagsTable.svelte";

  let {
    pinnedWay,
    drawProblemDetails,
    showProblemDetails = $bindable(),
  }: {
    pinnedWay: Feature<LineString, WayProps>;
    drawProblemDetails: FeatureCollection<
      Geometry,
      { label: string; color: string }
    >;
    showProblemDetails: boolean;
  } = $props();

  let loading = $state("");
  let recentlyAddedTags = $state<Set<string>>(new Set());
  let lastWayId = $state<number | null>(null);

  // Clear highlighting when a different way is selected
  $effect(() => {
    const currentWayId = pinnedWay.properties.id;
    if (lastWayId !== null && lastWayId !== currentWayId) {
      recentlyAddedTags = new Set();
    }
    lastWayId = currentWayId;
  });

  async function updateTags(removeKeys: string[], addTags: Array<string[]>) {
    // Only track tags that were explicitly set by the user
    recentlyAddedTags = new Set(addTags.map(([key]) => key));

    loading = "Setting tags";
    await refreshLoadingScreen();

    try {
      $backend!.editSetTags(
        BigInt(pinnedWay.properties.id),
        removeKeys,
        addTags,
      );
      $mutationCounter++;
    } finally {
      loading = "";
    }
  }

  const normalizedSidewalkTags = $derived(
    $backend
      ? (JSON.parse(
          $backend.normalizeSidewalkTags(BigInt(pinnedWay.properties.id)),
        ) as { left?: string; right?: string; both?: string })
      : { left: undefined, right: undefined, both: undefined },
  );
</script>

<Loading {loading} />

<div class="card mb-5">
  <div class="card-header">
    <a
      href="https://www.openstreetmap.org/way/{pinnedWay.properties.id}/history"
      target="_blank"
    >
      Way {pinnedWay.properties.id}
    </a>
    (
    <a
      href="https://www.openstreetmap.org/edit?way={pinnedWay.properties.id}"
      target="_blank"
      title="Edit way"
    >
      <i class="fa-solid fa-pencil"></i>
    </a>
    ) : {kindLabels[pinnedWay.properties.kind]}
  </div>

  <div class="card-body">
    <Problems
      problems={pinnedWay.properties.problems}
      {drawProblemDetails}
      bind:showProblemDetails
    />

    {#if pinnedWay.properties.kind.startsWith("Road")}
      <CenterlineTagActions
        {normalizedSidewalkTags}
        {updateTags}
        currentTags={pinnedWay.properties.tags}
      />
    {:else if ["footway", "path", "cycleway"].includes(pinnedWay.properties.tags.highway ?? "")}
      <SidepathTagActions
        {updateTags}
        highway={pinnedWay.properties.tags.highway as
          | "footway"
          | "path"
          | "cycleway"}
        currentTags={pinnedWay.properties.tags}
      />
    {/if}

    <CurrentTagsTable
      tags={pinnedWay.properties.tags}
      {recentlyAddedTags}
      {updateTags}
    />

    {#if $debugMode}
      <p>Nodes: {pinnedWay.properties.node_ids.join(", ")}</p>
    {/if}
  </div>
</div>
