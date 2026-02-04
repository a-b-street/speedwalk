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

  // Extract existing sidewalk tags from way properties
  function getExistingSidewalkTags(
    tags: Record<string, string>,
  ): Array<[string, string]> {
    return Object.entries(tags)
      .filter(([key]) => key.startsWith("sidewalk"))
      .map(([key, value]) => [key, value] as [string, string]);
  }

  // Handle tag conflicts (sidewalk:both vs sidewalk:left/right)
  function resolveTagConflicts(
    tagMap: Map<string, string>,
    isSettingBoth: boolean,
    isSettingLeftOrRight: boolean,
  ): void {
    if (isSettingBoth) {
      tagMap.delete("sidewalk:left");
      tagMap.delete("sidewalk:right");
      tagMap.delete("sidewalk");
    } else if (isSettingLeftOrRight) {
      tagMap.delete("sidewalk:both");
      tagMap.delete("sidewalk");
    }
  }

  // Migrate from sidewalk:both or legacy sidewalk=separate to individual tags
  function migrateTags(
    tagMap: Map<string, string>,
    existingBothValue: string | undefined,
    hasLegacySeparate: boolean,
    isSettingLeft: boolean,
    isSettingRight: boolean,
  ): void {
    if (!isSettingLeft && !isSettingRight) return;

    if (existingBothValue) {
      // Migrating from sidewalk:both to individual tags
      if (isSettingLeft) {
        tagMap.set("sidewalk:right", existingBothValue);
      } else if (isSettingRight) {
        tagMap.set("sidewalk:left", existingBothValue);
      }
    } else if (hasLegacySeparate) {
      // Migrating from legacy sidewalk=separate to individual tags
      if (isSettingLeft) {
        const leftValue = tagMap.get("sidewalk:left");
        if (leftValue) {
          tagMap.set("sidewalk:right", leftValue);
        }
      } else if (isSettingRight) {
        const rightValue = tagMap.get("sidewalk:right");
        if (rightValue) {
          tagMap.set("sidewalk:left", rightValue);
        }
      }
    }
  }

  // Merge new tags with existing sidewalk tags, handling conflicts and migrations
  function mergeSidewalkTags(
    existingTags: Record<string, string>,
    newTags: Array<string[]>,
  ): Array<string[]> {
    const existingSidewalkTags = getExistingSidewalkTags(existingTags);
    const existingBothValue = existingTags["sidewalk:both"];
    const hasLegacySeparate = existingTags["sidewalk"] === "separate";

    // Create a map to merge tags (new tags override existing ones)
    const tagMap = new Map<string, string>();
    for (const [key, value] of existingSidewalkTags) {
      tagMap.set(key, value);
    }

    // Determine which new tags are being set
    const newTagKeys = new Set(newTags.map(([key]) => key));
    const isSettingBoth = newTagKeys.has("sidewalk:both");
    const isSettingLeft = newTagKeys.has("sidewalk:left");
    const isSettingRight = newTagKeys.has("sidewalk:right");
    const isSettingLeftOrRight = isSettingLeft || isSettingRight;

    // Handle conflicts
    resolveTagConflicts(tagMap, isSettingBoth, isSettingLeftOrRight);

    // Add new tags
    for (const [key, value] of newTags) {
      tagMap.set(key, value);
    }

    // Handle migrations
    migrateTags(
      tagMap,
      existingBothValue,
      hasLegacySeparate,
      isSettingLeft,
      isSettingRight,
    );

    const leftValue = tagMap.get("sidewalk:left");
    const rightValue = tagMap.get("sidewalk:right");
    if (leftValue && rightValue && leftValue === rightValue) {
      tagMap.set("sidewalk:both", leftValue);
      tagMap.delete("sidewalk:left");
      tagMap.delete("sidewalk:right");
    }

    return Array.from(tagMap.entries());
  }

  async function setTags(tags: Array<string[]>) {
    const mergedTags = mergeSidewalkTags(pinnedWay.properties.tags, tags);
    
    // Only track tags that were explicitly set by the user
    recentlyAddedTags = new Set(tags.map(([key]) => key));

    loading = "Setting tags";
    await refreshLoadingScreen();

    try {
      $backend!.editSetTags(BigInt(pinnedWay.properties.id), mergedTags);
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
      <CenterlineTagActions {normalizedSidewalkTags} {setTags} />
    {:else if ["footway", "path", "cycleway"].includes(pinnedWay.properties.tags.highway ?? "")}
      <SidepathTagActions
        {setTags}
        highway={pinnedWay.properties.tags.highway as
          | "footway"
          | "path"
          | "cycleway"}
      />
    {/if}

    <CurrentTagsTable tags={pinnedWay.properties.tags} {recentlyAddedTags} />

    {#if $debugMode}
      <p>Nodes: {pinnedWay.properties.node_ids.join(", ")}</p>
    {/if}
  </div>
</div>
