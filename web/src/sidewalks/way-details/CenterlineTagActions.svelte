<script lang="ts">
  import { siteColorRgba } from "../index";

  let {
    normalizedSidewalkTags,
    updateTags,
    currentTags,
  }: {
    normalizedSidewalkTags: { left?: string; right?: string; both?: string };
    updateTags: (
      removeKeys: string[],
      addTags: Array<string[]>,
    ) => Promise<void>;
    currentTags: Record<string, string>;
  } = $props();

  function getSidewalkTagKeysToRemove(): string[] {
    return Object.keys(currentTags).filter((key) => key.startsWith("sidewalk"));
  }

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

  function handleSidewalkTags(newTags: Array<string[]>): Array<string[]> {
    return mergeSidewalkTags(currentTags, newTags);
  }

  function getHighlightedCell(
    row: "left" | "right",
    column: "yes" | "no" | "separate",
  ): "active" | "both-highlight" | null {
    const value = normalizedSidewalkTags[row];
    if (value === column) {
      return "active";
    }
    if (normalizedSidewalkTags.both && normalizedSidewalkTags.both === column) {
      return "both-highlight";
    }
    return null;
  }

  function getTagsForShortcut(key: string): Array<string[]> | null {
    const shortcutMap: Record<string, Array<string[]>> = {
      q: [["sidewalk:left", "yes"]],
      a: [["sidewalk:left", "no"]],
      y: [["sidewalk:left", "separate"]],
      w: [["sidewalk:right", "yes"]],
      s: [["sidewalk:right", "no"]],
      x: [["sidewalk:right", "separate"]],
    };
    return shortcutMap[key] || null;
  }

  async function onKeyDown(e: KeyboardEvent) {
    const tags = getTagsForShortcut(e.key.toLowerCase());
    if (tags) {
      const mergedTags = handleSidewalkTags(tags);
      await updateTags(getSidewalkTagKeysToRemove(), mergedTags);
    }
  }
</script>

<svelte:window onkeydown={onKeyDown} />

<table class="table table-bordered">
  <thead>
    <tr>
      <th style="background-color: {siteColorRgba('left', 0.3)};">Left</th>
      <th style="background-color: {siteColorRgba('right', 0.3)};">Right</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td
        class:table-active={getHighlightedCell("left", "yes") === "active"}
        style:background-color={getHighlightedCell("left", "yes") ===
        "both-highlight"
          ? siteColorRgba("left", 0.15)
          : ""}
      >
        <button
          class="btn btn-sm btn-secondary w-100"
          class:disabled={getHighlightedCell("left", "yes") === "active"}
          onclick={() => {
            const mergedTags = handleSidewalkTags([["sidewalk:left", "yes"]]);
            updateTags(getSidewalkTagKeysToRemove(), mergedTags);
          }}
        >
          Yes <kbd>q</kbd>
        </button>
      </td>
      <td
        class:table-active={getHighlightedCell("right", "yes") === "active"}
        style:background-color={getHighlightedCell("right", "yes") ===
        "both-highlight"
          ? siteColorRgba("right", 0.15)
          : ""}
      >
        <button
          class="btn btn-sm btn-secondary w-100"
          class:disabled={getHighlightedCell("right", "yes") === "active"}
          onclick={() => {
            const mergedTags = handleSidewalkTags([["sidewalk:right", "yes"]]);
            updateTags(getSidewalkTagKeysToRemove(), mergedTags);
          }}
        >
          Yes <kbd>w</kbd>
        </button>
      </td>
    </tr>
    <tr>
      <td
        class:table-active={getHighlightedCell("left", "no") === "active"}
        style:background-color={getHighlightedCell("left", "no") ===
        "both-highlight"
          ? siteColorRgba("left", 0.15)
          : ""}
      >
        <button
          class="btn btn-sm btn-secondary w-100"
          class:disabled={getHighlightedCell("left", "no") === "active"}
          onclick={() => {
            const mergedTags = handleSidewalkTags([["sidewalk:left", "no"]]);
            updateTags(getSidewalkTagKeysToRemove(), mergedTags);
          }}
        >
          No <kbd>a</kbd>
        </button>
      </td>
      <td
        class:table-active={getHighlightedCell("right", "no") === "active"}
        style:background-color={getHighlightedCell("right", "no") ===
        "both-highlight"
          ? siteColorRgba("right", 0.15)
          : ""}
      >
        <button
          class="btn btn-sm btn-secondary w-100"
          class:disabled={getHighlightedCell("right", "no") === "active"}
          onclick={() => {
            const mergedTags = handleSidewalkTags([["sidewalk:right", "no"]]);
            updateTags(getSidewalkTagKeysToRemove(), mergedTags);
          }}
        >
          No <kbd>s</kbd>
        </button>
      </td>
    </tr>
    <tr>
      <td
        class:table-active={getHighlightedCell("left", "separate") === "active"}
        style:background-color={getHighlightedCell("left", "separate") ===
        "both-highlight"
          ? `rgba(255, 105, 180, 0.15)`
          : ""}
      >
        <button
          class="btn btn-sm btn-secondary w-100"
          class:disabled={getHighlightedCell("left", "separate") === "active"}
          onclick={() => {
            const mergedTags = handleSidewalkTags([
              ["sidewalk:left", "separate"],
            ]);
            updateTags(getSidewalkTagKeysToRemove(), mergedTags);
          }}
        >
          Separate <kbd>y</kbd>
        </button>
      </td>
      <td
        class:table-active={getHighlightedCell("right", "separate") ===
          "active"}
        style:background-color={getHighlightedCell("right", "separate") ===
        "both-highlight"
          ? siteColorRgba("right", 0.15)
          : ""}
      >
        <button
          class="btn btn-sm btn-secondary w-100"
          class:disabled={getHighlightedCell("right", "separate") === "active"}
          onclick={() => {
            const mergedTags = handleSidewalkTags([
              ["sidewalk:right", "separate"],
            ]);
            updateTags(getSidewalkTagKeysToRemove(), mergedTags);
          }}
        >
          Separate <kbd>x</kbd>
        </button>
      </td>
    </tr>
  </tbody>
</table>
