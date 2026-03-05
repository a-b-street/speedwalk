<script lang="ts">
  import LegendListWrapper from "./LegendListWrapper.svelte";
  import LegendListItem from "./LegendListItem.svelte";

  export interface LegendItem {
    label: string;
    color: string;
    swatchClass?: "rectangle" | "circle";
  }

  interface Props {
    items: LegendItem[];
    swatchClass?: "rectangle" | "circle";
  }

  let { items, swatchClass = "rectangle" }: Props = $props();
</script>

<LegendListWrapper>
  {#snippet children()}
    {#each items as item}
      <LegendListItem
        color={item.color}
        swatchClass={item.swatchClass ?? swatchClass}
      >
        {#snippet children()}
          {item.label}
        {/snippet}
      </LegendListItem>
    {/each}
  {/snippet}
</LegendListWrapper>
