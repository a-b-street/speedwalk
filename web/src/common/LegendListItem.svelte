<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    color: string;
    swatchClass?: "rectangle" | "circle";
    children?: Snippet;
  }

  let { color, swatchClass = "rectangle", children }: Props = $props();
</script>

<li class="legend-item">
  <span class="color-swatch {swatchClass}" style:background={color}></span>
  {#if children}
    {@render children()}
  {/if}
</li>

<style>
  .legend-item {
    display: flex;
    align-items: center;
    gap: 6px;
    line-height: 1.1;
  }
  .legend-item :global(.form-check) {
    margin: 0;
    padding: 0;
    min-height: 0;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 0.375rem;
  }
  .legend-item :global(.form-check-input) {
    margin-inline-start: 0;
    flex-shrink: 0;
  }
  .legend-item :global(.form-check-label) {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    min-width: 0;
  }
  .color-swatch {
    flex-shrink: 0;
    border: 1px solid #888;
  }
  .color-swatch.circle {
    height: 20px;
    width: 20px;
    border-radius: 10px;
  }
  .color-swatch.rectangle {
    height: 10px;
    width: 24px;
    border-radius: 1px;
  }
</style>
