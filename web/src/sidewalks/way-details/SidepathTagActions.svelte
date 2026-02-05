<script lang="ts">
  let {
    updateTags,
    highway,
    currentTags,
  }: {
    updateTags: (
      removeKeys: string[],
      addTags: Array<string[]>,
    ) => Promise<void>;
    highway: "footway" | "path" | "cycleway";
    currentTags: Record<string, string>;
  } = $props();

  function getRelevantRemovals(removes: string[]): string[] {
    return removes.filter((key) => key in currentTags);
  }

  const actions = $derived.by(() => {
    switch (highway) {
      case "path":
        return [
          { shortcut: "s", tags: [["is_sidepath", "yes"]], removes: [] },
          { shortcut: "n", tags: [["is_sidepath", "no"]], removes: [] },
          { shortcut: "c", tags: [["path", "crossing"]], removes: [] },
        ];
      case "footway":
        return [
          {
            shortcut: "s",
            tags: [["footway", "sidewalk"]],
            removes: ["is_sidepath"],
          },
          {
            shortcut: "n",
            tags: [["is_sidepath", "no"]],
            removes: ["footway"],
          },
          { shortcut: "c", tags: [["footway", "crossing"]], removes: [] },
        ];
      case "cycleway":
        return [
          { shortcut: "s", tags: [["is_sidepath", "yes"]], removes: [] },
          { shortcut: "n", tags: [["is_sidepath", "no"]], removes: [] },
          { shortcut: "c", tags: [["cycleway", "crossing"]], removes: [] },
        ];
    }
  });

  async function onKeyDown(e: KeyboardEvent) {
    const key = e.key.toLowerCase();
    const action = actions.find((a) => a.shortcut === key);
    if (action) {
      await updateTags(
        action.removes,
        action.tags.map((p) => [...p]),
      );
    }
  }
</script>

<svelte:window onkeydown={onKeyDown} />

<ul class="list-group list-group-flush mb-0">
  {#each actions as action}
    {@const relevantRemovals = getRelevantRemovals(action.removes)}
    <li class="list-group-item px-0 py-1 border-0">
      <button
        type="button"
        class="btn btn-secondary w-100"
        onclick={() =>
          updateTags(
            action.removes,
            action.tags.map((p) => [...p]),
          )}
      >
        {action.tags.map((pair) => `${pair[0]}=${pair[1]}`).join(", ")}
        <kbd>{action.shortcut}</kbd>
      </button>
      {#if relevantRemovals.length > 0}
        <div class="text-secondary small mt-1">
          Removes: {relevantRemovals.join(", ")}
        </div>
      {/if}
    </li>
  {/each}
</ul>
