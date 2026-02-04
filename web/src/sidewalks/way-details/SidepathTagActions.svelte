<script lang="ts">
  let {
    setTags,
    highway,
  }: {
    setTags: (tags: Array<string[]>) => Promise<void>;
    highway: "footway" | "path" | "cycleway";
  } = $props();

  const actions = $derived.by(() => {
    switch (highway) {
      case "path":
        return [
          { shortcut: "s", tags: [["is_sidepath", "yes"]] },
          { shortcut: "n", tags: [["is_sidepath", "no"]] },
          { shortcut: "c", tags: [["path", "crossing"]] },
        ];
      case "footway":
        return [
          { shortcut: "s", tags: [["footway", "sidewalk"]] },
          { shortcut: "n", tags: [["is_sidepath", "no"]] },
          { shortcut: "c", tags: [["footway", "crossing"]] },
        ];
      case "cycleway":
        return [
          { shortcut: "s", tags: [["is_sidepath", "yes"]] },
          { shortcut: "n", tags: [["is_sidepath", "no"]] },
          { shortcut: "c", tags: [["cycleway", "crossing"]] },
        ];
    }
  });

  async function onKeyDown(e: KeyboardEvent) {
    const key = e.key.toLowerCase();
    const action = actions.find((a) => a.shortcut === key);
    if (action) {
      await setTags(action.tags.map((p) => [...p]));
    }
  }
</script>

<svelte:window onkeydown={onKeyDown} />

<ul class="list-group list-group-flush mb-0">
  {#each actions as action}
    <li class="list-group-item px-0 py-1 border-0">
      <button
        type="button"
        class="btn btn-secondary w-100"
        onclick={() => setTags(action.tags.map((p) => [...p]))}
      >
        {action.tags.map((pair) => `${pair[0]}=${pair[1]}`).join(", ")}
        <kbd>{action.shortcut}</kbd>
      </button>
    </li>
  {/each}
</ul>
