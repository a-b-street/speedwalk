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
  import {
    kindLabels,
    siteColorRgba,
    type WayProps,
  } from "./";

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

  // Extract existing sidewalk tags from way properties
  function getExistingSidewalkTags(tags: Record<string, string>): Array<[string, string]> {
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
    migrateTags(tagMap, existingBothValue, hasLegacySeparate, isSettingLeft, isSettingRight);

    return Array.from(tagMap.entries());
  }

  async function setTags(tags: Array<string[]>) {
    const mergedTags = mergeSidewalkTags(pinnedWay.properties.tags, tags);

    loading = "Setting tags";
    await refreshLoadingScreen();

    try {
      $backend!.editSetTags(BigInt(pinnedWay.properties.id), mergedTags);
      $mutationCounter++;
    } finally {
      loading = "";
    }
  }


  // Get highlighted cell state
  function getHighlightedCell(
    normalized: { left?: string; right?: string; both?: string },
    row: "left" | "right" | "both",
    column: "yes" | "no" | "separate",
  ): "active" | "both-highlight" | null {
    if (row === "both") {
      // For "both" row, check if sidewalk:both has this value
      if (normalized.both === column) {
        return "active";
      }
      return null;
    }

    // For left/right rows
    const value = normalized[row];
    if (value === column) {
      return "active";
    }

    // Check if sidewalk:both is set - if so, highlight both left and right in lighter color
    if (normalized.both) {
      // Map sidewalk:both values to what they mean for left/right
      if (
        (normalized.both === "yes" && column === "yes") ||
        (normalized.both === "no" && column === "no") ||
        (normalized.both === "separate" && column === "separate")
      ) {
        return "both-highlight";
      }
    }

    return null;
  }


  let footwayFixTagChoices = [
    [["footway", "sidewalk"]],
    [["footway", "crossing"]],
  ];

  function getSortedTags(tags: Record<string, string>): Array<[string, string]> {
    const entries = Object.entries(tags);
    const sidewalkTags = entries.filter(([key]) => key.toLowerCase().startsWith("sidewalk"));
    const otherTags = entries.filter(([key]) => !key.toLowerCase().startsWith("sidewalk"));

    // Sort sidewalk tags A-Z
    sidewalkTags.sort(([a], [b]) => a.localeCompare(b));
    // Sort other tags A-Z
    otherTags.sort(([a], [b]) => a.localeCompare(b));

    // Return sidewalk tags first, then other tags
    return [...sidewalkTags, ...otherTags] as Array<[string, string]>;
  }

  function getTagsForShortcut(key: string): Array<string[]> | null {
    // Row 1 (Yes): q=left, w=right, e=both
    // Row 2 (No): a=left, s=right, d=both
    // Row 3 (Separate): y=left, x=right, c=both
    const shortcutMap: Record<string, Array<string[]>> = {
      q: [["sidewalk:left", "yes"]],
      w: [["sidewalk:right", "yes"]],
      e: [["sidewalk:both", "yes"]],
      a: [["sidewalk:left", "no"]],
      s: [["sidewalk:right", "no"]],
      d: [["sidewalk:both", "no"]],
      y: [["sidewalk:left", "separate"]],
      x: [["sidewalk:right", "separate"]],
      c: [["sidewalk:both", "separate"]],
    };
    return shortcutMap[key.toLowerCase()] || null;
  }

  async function onKeyDown(e: KeyboardEvent) {
    if (!pinnedWay.properties.kind.startsWith("Road")) {
      // Handle footway shortcuts (old behavior)
      if (pinnedWay.properties.tags.highway == "footway") {
        let n = parseInt(e.key);
        if (Number.isInteger(n) && n <= footwayFixTagChoices.length) {
          await setTags(footwayFixTagChoices[n - 1]);
        }
      }
      return;
    }

    // Only process lowercase keys for shortcuts
    const key = e.key.toLowerCase();
    const tags = getTagsForShortcut(key);
    if (tags) {
      await setTags(tags);
    }
  }
</script>

<svelte:window onkeydown={onKeyDown} />

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
      title="Edit way"
    >
      <i class="fa-solid fa-pencil"></i>
    </a>
    ) : {kindLabels[pinnedWay.properties.kind]}
  </div>

  <div class="card-body">
    {#if pinnedWay.properties.problems.length}
      {@const headerProblem = pinnedWay.properties.problems.find(p => p.note === "possible separate sidewalk near way without it tagged") || pinnedWay.properties.problems[0]}
      {@const remainingProblems = pinnedWay.properties.problems.filter(p => p.note !== headerProblem.note)}
      <div class="alert alert-warning">
        <h5 class="alert-heading d-flex align-items-center">
          <div class="flex-shrink-0 me-2">
            <i class="fa-solid fa-triangle-exclamation"></i>
          </div>
          <div class="flex-grow-1">
            {headerProblem.note}
          </div>
        </h5>
        {#if remainingProblems.length}
          {#each remainingProblems as problem}
            <p class="mb-0">{problem.note}</p>
          {/each}
        {/if}

        {#if drawProblemDetails.features.length}
          <Checkbox bind:checked={showProblemDetails}>
            Highlight problem on map
          </Checkbox>

          {#if showProblemDetails}
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
        {/if}
      </div>
    {/if}

    {#if pinnedWay.properties.kind.startsWith("Road")}
      {@const normalized = $backend ? (JSON.parse($backend.normalizeSidewalkTags(BigInt(pinnedWay.properties.id))) as { left?: string; right?: string; both?: string }) : { left: undefined, right: undefined, both: undefined }}
      <table class="table table-bordered">
        <thead>
          <tr>
            <th style="background-color: {siteColorRgba("left", 0.3)};">
              Left
            </th>
            <th style="background-color: {siteColorRgba("right", 0.3)};">
              Right
            </th>
            <th>Both</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td
              class:table-active={getHighlightedCell(normalized, "left", "yes") === "active"}
              style:background-color={getHighlightedCell(normalized, "left", "yes") === "both-highlight" ? siteColorRgba("left", 0.15) : ""}
            >
              <button
                class="btn btn-sm btn-secondary w-100 position-relative"
                class:disabled={getHighlightedCell(normalized, "left", "yes") === "active"}
                onclick={() => setTags([["sidewalk:left", "yes"]])}
              >
                <kbd class="shortcut-badge">q</kbd> Yes
              </button>
            </td>
            <td
              class:table-active={getHighlightedCell(normalized, "right", "yes") === "active"}
              style:background-color={getHighlightedCell(normalized, "right", "yes") === "both-highlight" ? siteColorRgba("right", 0.15) : ""}
            >
              <button
                class="btn btn-sm btn-secondary w-100 position-relative"
                class:disabled={getHighlightedCell(normalized, "right", "yes") === "active"}
                onclick={() => setTags([["sidewalk:right", "yes"]])}
              >
                <kbd class="shortcut-badge">w</kbd> Yes
              </button>
            </td>
            <td
              class:table-active={getHighlightedCell(normalized, "both", "yes") === "active"}
            >
              <button
                class="btn btn-sm btn-secondary w-100 position-relative"
                class:disabled={getHighlightedCell(normalized, "both", "yes") === "active"}
                onclick={() => setTags([["sidewalk:both", "yes"]])}
              >
                <kbd class="shortcut-badge">e</kbd> Yes
              </button>
            </td>
          </tr>
          <tr>
            <td
              class:table-active={getHighlightedCell(normalized, "left", "no") === "active"}
              style:background-color={getHighlightedCell(normalized, "left", "no") === "both-highlight" ? siteColorRgba("left", 0.15) : ""}
            >
              <button
                class="btn btn-sm btn-secondary w-100 position-relative"
                class:disabled={getHighlightedCell(normalized, "left", "no") === "active"}
                onclick={() => setTags([["sidewalk:left", "no"]])}
              >
                <kbd class="shortcut-badge">a</kbd> No
              </button>
            </td>
            <td
              class:table-active={getHighlightedCell(normalized, "right", "no") === "active"}
              style:background-color={getHighlightedCell(normalized, "right", "no") === "both-highlight" ? siteColorRgba("right", 0.15) : ""}
            >
              <button
                class="btn btn-sm btn-secondary w-100 position-relative"
                class:disabled={getHighlightedCell(normalized, "right", "no") === "active"}
                onclick={() => setTags([["sidewalk:right", "no"]])}
              >
                <kbd class="shortcut-badge">s</kbd> No
              </button>
            </td>
            <td
              class:table-active={getHighlightedCell(normalized, "both", "no") === "active"}
            >
              <button
                class="btn btn-sm btn-secondary w-100 position-relative"
                class:disabled={getHighlightedCell(normalized, "both", "no") === "active"}
                onclick={() => setTags([["sidewalk:both", "no"]])}
              >
                <kbd class="shortcut-badge">d</kbd> No
              </button>
            </td>
          </tr>
          <tr>
            <td
              class:table-active={getHighlightedCell(normalized, "left", "separate") === "active"}
              style:background-color={getHighlightedCell(normalized, "left", "separate") === "both-highlight" ? `rgba(255, 105, 180, 0.15)` : ""}
            >
              <button
                class="btn btn-sm btn-secondary w-100 position-relative"
                class:disabled={getHighlightedCell(normalized, "left", "separate") === "active"}
                onclick={() => setTags([["sidewalk:left", "separate"]])}
              >
                <kbd class="shortcut-badge">y</kbd> Separate
              </button>
            </td>
            <td
              class:table-active={getHighlightedCell(normalized, "right", "separate") === "active"}
              style:background-color={getHighlightedCell(normalized, "right", "separate") === "both-highlight" ? siteColorRgba("right", 0.15) : ""}
            >
              <button
                class="btn btn-sm btn-secondary w-100 position-relative"
                class:disabled={getHighlightedCell(normalized, "right", "separate") === "active"}
                onclick={() => setTags([["sidewalk:right", "separate"]])}
              >
                <kbd class="shortcut-badge">x</kbd> Separate
              </button>
            </td>
            <td
              class:table-active={getHighlightedCell(normalized, "both", "separate") === "active"}
            >
              <button
                class="btn btn-sm btn-secondary w-100 position-relative"
                class:disabled={getHighlightedCell(normalized, "both", "separate") === "active"}
                onclick={() => setTags([["sidewalk:both", "separate"]])}
              >
                <kbd class="shortcut-badge">c</kbd> Separate
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    {:else if pinnedWay.properties.tags.highway == "footway"}
      <u>Set these tags</u>

      {#each footwayFixTagChoices.entries() as [idx, tags]}
        <div>
          <button class="btn btn-secondary mb-1" onclick={() => setTags(tags)}>
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
        {#each getSortedTags(pinnedWay.properties.tags) as [key, value]}
          <tr class:table-active={key.toLowerCase().includes("sidewalk")}>
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

<style>
  .shortcut-badge {
    position: absolute;
    top: -8px;
    left: -8px;
    background-color: #6c757d;
    color: white;
    border-radius: 10px;
    min-width: 18px;
    height: 18px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-size: 8px;
    font-weight: bold;
    padding: 0 4px;
    line-height: 1;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
    z-index: 1;
    white-space: nowrap;
  }


  button:disabled .shortcut-badge {
    opacity: 0.6;
  }

  :global(.alert .color-swatch) {
    flex-shrink: 0;
  }
</style>
