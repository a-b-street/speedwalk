<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    children,
  }: {
    children: Snippet;
  } = $props();
</script>

<div style="display: flex; align-items: center; gap: 6px;">
  {@render children()}
  <i
    class="fa-solid fa-hard-drive"
    style="font-size: 0.85em; color: #6c757d; cursor: help; flex-shrink: 0;"
    title="This setting is saved locally"
    aria-label="This setting is saved locally"
  ></i>
</div>
