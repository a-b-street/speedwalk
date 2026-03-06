<script lang="ts">
  import CollapsibleCard from "./CollapsibleCard.svelte";
  import NetworkFilter from "./NetworkFilter.svelte";
  import { LocalStorageWrapper } from "svelte-utils";
</script>

<CollapsibleCard open={true}>
  {#snippet header()}
    <LocalStorageWrapper>
      <span>Filter network</span>
    </LocalStorageWrapper>
  {/snippet}
  {#snippet body()}
    <NetworkFilter />
  {/snippet}
</CollapsibleCard>
