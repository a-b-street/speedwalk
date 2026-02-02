<script lang="ts">
  let {
    setTags,
  }: {
    setTags: (tags: Array<string[]>) => Promise<void>;
  } = $props();

  const tagChoices = [[["footway", "sidewalk"]], [["footway", "crossing"]]];

  async function onKeyDown(e: KeyboardEvent) {
    const n = parseInt(e.key, 10);
    if (Number.isInteger(n) && n >= 1 && n <= tagChoices.length) {
      await setTags(tagChoices[n - 1]);
    }
  }
</script>

<svelte:window onkeydown={onKeyDown} />

<u>Set these tags</u>

{#each tagChoices.entries() as [idx, tags]}
  <div>
    <button class="btn btn-secondary mb-1" onclick={() => setTags(tags)}>
      <kbd>{idx + 1}</kbd>
      {tags.map((pair) => `${pair[0]} = ${pair[1]}`).join(", ")}
    </button>
  </div>
{/each}
