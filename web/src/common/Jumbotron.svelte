<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    title: string;
    lead?: string;
    children?: Snippet;
  }
  let { title, lead, children }: Props = $props();
</script>

<div class="jumbotron p-3 rounded bg-light mb-5 pb-1">
  <h1 class="h5">{title}</h1>
  {#if lead}
    <p>{lead}</p>
  {/if}
  {#if children}
    <div class="pb-2">
      {@render children?.()}
    </div>
  {/if}
</div>

<style>
  .jumbotron {
    border: 1px solid rgba(0, 0, 0, 0.1);
  }
</style>
