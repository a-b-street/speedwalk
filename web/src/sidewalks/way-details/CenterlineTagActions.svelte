<script lang="ts">
  import { siteColorRgba } from "../index";

  let {
    normalizedSidewalkTags,
    setTags,
  }: {
    normalizedSidewalkTags: { left?: string; right?: string; both?: string };
    setTags: (tags: Array<string[]>) => Promise<void>;
  } = $props();

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
      await setTags(tags);
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
          onclick={() => setTags([["sidewalk:left", "yes"]])}
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
          onclick={() => setTags([["sidewalk:right", "yes"]])}
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
          onclick={() => setTags([["sidewalk:left", "no"]])}
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
          onclick={() => setTags([["sidewalk:right", "no"]])}
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
          onclick={() => setTags([["sidewalk:left", "separate"]])}
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
          onclick={() => setTags([["sidewalk:right", "separate"]])}
        >
          Separate <kbd>x</kbd>
        </button>
      </td>
    </tr>
  </tbody>
</table>
