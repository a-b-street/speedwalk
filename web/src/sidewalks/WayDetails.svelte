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
  import { kindLabels, siteColorRgba, type WayProps } from "./";

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

  // Reset keyboard shortcut state when way changes
  let currentWayId = $state(pinnedWay.properties.id);
  $effect(() => {
    if (pinnedWay.properties.id !== currentWayId) {
      // Way changed - reset all shortcut state
      keySequence = "";
      pendingTags = [];
      lastCompletedSequence = "";
      if (keyTimer) {
        clearTimeout(keyTimer);
        keyTimer = null;
      }
      if (pendingTagsTimer) {
        clearTimeout(pendingTagsTimer);
        pendingTagsTimer = null;
      }
      currentWayId = pinnedWay.properties.id;
    }
  });

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

  // Normalize sidewalk tags to determine current state
  function normalizeSidewalkTags(tags: Record<string, string>) {
    const normalized: {
      left?: string;
      right?: string;
      both?: string;
    } = {};

    // Check for sidewalk:both tag (highest priority)
    if (tags["sidewalk:both"]) {
      normalized.both = tags["sidewalk:both"];
    }

    // Check for sidewalk tag (legacy format)
    // Only process if sidewalk:both doesn't exist (to avoid conflicts)
    if (tags["sidewalk"] && !tags["sidewalk:both"]) {
      const value = tags["sidewalk"];
      if (value === "left") {
        normalized.left = "yes";
        normalized.right = "no";
      } else if (value === "right") {
        normalized.left = "no";
        normalized.right = "yes";
      } else if (value === "both") {
        normalized.both = "yes";
      } else if (value === "separate") {
        normalized.both = "separate";
      } else if (value === "no" || value === "none") {
        normalized.both = "no";
      }
    }

    // Direct sidewalk:left and sidewalk:right tags override normalized values
    if (tags["sidewalk:left"]) {
      normalized.left = tags["sidewalk:left"];
    }
    if (tags["sidewalk:right"]) {
      normalized.right = tags["sidewalk:right"];
    }

    return normalized;
  }

  // Get highlighted cell state
  function getHighlightedCell(
    normalized: ReturnType<typeof normalizeSidewalkTags>,
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

  // Keyboard shortcut state
  let keySequence = $state("");
  let keyTimer: ReturnType<typeof setTimeout> | null = null;
  // Accumulate tag arrays from multiple shortcuts: Array<Array<[key, value]>>
  let pendingTags: Array<Array<string[]>> = $state([]);
  let pendingTagsTimer: ReturnType<typeof setTimeout> | null = null;
  let lastCompletedSequence = $state("");

  function getTagsForShortcut(seq: string): Array<string[]> {
    // Determine row and column from sequence
    // Digit determines row (1=yes, 2=no, 3=separate)
    // Repetitions determine column (1=left, 2=right, 3=both)
    const rowDigit = parseInt(seq[0]);
    const count = seq.length;

    if (rowDigit === 1) {
      // Yes row
      if (count === 1) {
        return [["sidewalk:left", "yes"]];
      } else if (count === 2) {
        return [["sidewalk:right", "yes"]];
      } else if (count >= 3) {
        return [["sidewalk:both", "yes"]];
      }
    } else if (rowDigit === 2) {
      // No row
      if (count === 1) {
        return [["sidewalk:left", "no"]];
      } else if (count === 2) {
        return [["sidewalk:right", "no"]];
      } else if (count >= 3) {
        return [["sidewalk:both", "no"]];
      }
    } else if (rowDigit === 3) {
      // Separate row
      if (count === 1) {
        return [["sidewalk:left", "separate"]];
      } else if (count === 2) {
        return [["sidewalk:right", "separate"]];
      } else if (count >= 3) {
        return [["sidewalk:both", "separate"]];
      }
    }
    return [];
  }

  async function executePendingTags() {
    if (pendingTags.length > 0) {
      // Merge all accumulated tags into a single array
      // Later tags override earlier ones for the same key
      const tagMap = new Map<string, string>();
      for (const tagArray of pendingTags) {
        for (const [key, value] of tagArray) {
          tagMap.set(key, value);
        }
      }
      // Convert Map entries to Array<[string, string]> format expected by setTags
      const tagsToSet: Array<string[]> = Array.from(tagMap.entries()).map(
        ([key, value]) => [key, value],
      );
      pendingTags = [];
      lastCompletedSequence = ""; // Reset tracking after execution
      await setTags(tagsToSet);
    }
  }

  function addPendingTags(tags: Array<string[]>) {
    // Remove conflicting tags (same key) before adding new ones
    // This allows users to "change their mind" - e.g., press 22 then 2 to override
    const newKeys = new Set(tags.map(([key]) => key));
    pendingTags = pendingTags.filter((tagArray) =>
      tagArray.every(([key]) => !newKeys.has(key)),
    );

    // Add the new tags
    pendingTags.push(tags);

    // Clear existing timer
    if (pendingTagsTimer) {
      clearTimeout(pendingTagsTimer);
    }
    // Set timer to execute accumulated tags after a delay
    // This allows multiple shortcuts to accumulate before executing
    // Use a longer delay (1000ms) so sequences can accumulate even with pauses
    pendingTagsTimer = setTimeout(async () => {
      await executePendingTags();
      pendingTagsTimer = null;
    }, 1000); // Longer delay to allow multiple shortcuts to accumulate even with pauses
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

    const digit = parseInt(e.key);
    if (!Number.isInteger(digit) || digit < 1 || digit > 3) {
      // Execute pending sequence if any, then reset
      if (keySequence.length > 0) {
        const tags = getTagsForShortcut(keySequence);
        if (tags.length > 0) {
          addPendingTags(tags);
        }
      }
      // Execute accumulated tags immediately
      await executePendingTags();
      keySequence = "";
      if (keyTimer) {
        clearTimeout(keyTimer);
        keyTimer = null;
      }
      return;
    }

    // Check if this is a different digit than before
    if (keySequence.length > 0 && keySequence[0] !== e.key) {
      // Different digit - add previous sequence to pending tags (if not already added)
      if (keyTimer) {
        clearTimeout(keyTimer);
        keyTimer = null;
      }
      // Only add if this sequence hasn't been completed yet
      if (keySequence !== lastCompletedSequence) {
        const tags = getTagsForShortcut(keySequence);
        if (tags.length > 0) {
          addPendingTags(tags);
          lastCompletedSequence = keySequence;
        }
      }
      keySequence = "";
    }

    // Add to sequence
    keySequence += e.key;

    // Clear existing timer
    if (keyTimer) {
      clearTimeout(keyTimer);
    }

    // Set timer to process after a short delay
    keyTimer = setTimeout(async () => {
      if (keySequence.length > 0) {
        const tags = getTagsForShortcut(keySequence);
        if (tags.length > 0) {
          addPendingTags(tags);
          lastCompletedSequence = keySequence;
        }
        keySequence = "";
      }
      keyTimer = null;
    }, 300); // 300ms delay to detect multi-digit sequences
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
      {@const normalized = normalizeSidewalkTags(pinnedWay.properties.tags)}
      <u>Current sidewalk tags</u>
      <ul>
        {#each Object.entries(pinnedWay.properties.tags) as [key, value]}
          {#if key.startsWith("sidewalk")}
            <li>{key} = {value}</li>
          {/if}
        {/each}
      </ul>

      <u>Set sidewalk tags</u>
      <table class="table table-bordered">
        <thead>
          <tr>
            <th>Tag</th>
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
            <td><strong>Yes</strong></td>
            <td
              class:table-active={getHighlightedCell(normalized, "left", "yes") === "active"}
              style:background-color={getHighlightedCell(normalized, "left", "yes") === "both-highlight" ? siteColorRgba("left", 0.15) : ""}
            >
              <button
                class="btn btn-sm btn-secondary w-100 position-relative"
                class:disabled={getHighlightedCell(normalized, "left", "yes") === "active"}
                onclick={() => setTags([["sidewalk:left", "yes"]])}
              >
                <kbd class="shortcut-badge">1</kbd> Yes
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
                <kbd class="shortcut-badge">11</kbd> Yes
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
                <kbd class="shortcut-badge">111</kbd> Yes
              </button>
            </td>
          </tr>
          <tr>
            <td><strong>No</strong></td>
            <td
              class:table-active={getHighlightedCell(normalized, "left", "no") === "active"}
              style:background-color={getHighlightedCell(normalized, "left", "no") === "both-highlight" ? siteColorRgba("left", 0.15) : ""}
            >
              <button
                class="btn btn-sm btn-secondary w-100 position-relative"
                class:disabled={getHighlightedCell(normalized, "left", "no") === "active"}
                onclick={() => setTags([["sidewalk:left", "no"]])}
              >
                <kbd class="shortcut-badge">2</kbd> No
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
                <kbd class="shortcut-badge">22</kbd> No
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
                <kbd class="shortcut-badge">222</kbd> No
              </button>
            </td>
          </tr>
          <tr>
            <td><strong>Separate</strong></td>
            <td
              class:table-active={getHighlightedCell(normalized, "left", "separate") === "active"}
              style:background-color={getHighlightedCell(normalized, "left", "separate") === "both-highlight" ? `rgba(255, 105, 180, 0.15)` : ""}
            >
              <button
                class="btn btn-sm btn-secondary w-100 position-relative"
                class:disabled={getHighlightedCell(normalized, "left", "separate") === "active"}
                onclick={() => setTags([["sidewalk:left", "separate"]])}
              >
                <kbd class="shortcut-badge">3</kbd> Separate
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
                <kbd class="shortcut-badge">33</kbd> Separate
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
                <kbd class="shortcut-badge">333</kbd> Separate
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
        {#each Object.entries(pinnedWay.properties.tags) as [key, value]}
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
</style>
