<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children?: Snippet;
  }

  let { children }: Props = $props();
</script>

<ul class="legend-list">
  {#if children}
    {@render children()}
  {/if}
</ul>

<style>
  .legend-list {
    list-style: none;
    margin: 0;
    padding: 0;
    padding-inline-start: 0;
    display: flex;
    flex-direction: column;
    align-items: stretch;
    gap: 0.5rem;
  }
</style>
