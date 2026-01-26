<script lang="ts">
  import CollapsibleCard from "../common/CollapsibleCard.svelte";
  import { Modal, Loading } from "svelte-utils";
  import {
    backend,
    mutationCounter,
    enabledBulkOps,
    refreshLoadingScreen,
  } from "../";
  import LocalStorageWrapper from "../common/LocalStorageWrapper.svelte";

  let { options }: { options: any } = $props();

  let show = $state(false);
  function enableOps() {
    $enabledBulkOps = true;
    show = false;
  }

  let loading = $state("");

  async function generateCrossings() {
    loading = "Generating missing crossings";
    await refreshLoadingScreen();
    try {
      $backend!.editGenerateMissingCrossings(options);
      $mutationCounter++;
    } catch (err) {
      window.alert(`Error: ${err}`);
    } finally {
      loading = "";
    }
  }
</script>

<Loading {loading} />

{#if $enabledBulkOps}
  <CollapsibleCard>
    {#snippet header()}Bulk operations{/snippet}
    {#snippet body()}
      <button class="btn btn-secondary" onclick={generateCrossings}>
        Generate imaginary crossings where they're missing
      </button>
    {/snippet}
  </CollapsibleCard>
{:else}
  <button class="btn btn-secondary" onclick={() => (show = true)}>
    Bulk operations
  </button>
{/if}

<Modal bind:show>
  <LocalStorageWrapper>
    <h2>Bulk operations</h2>
  </LocalStorageWrapper>

  <p>
    Speedwalk has some experimental features that can automatically generate
    crossing nodes on every arm of a junction where they're expected. This is
    intended <b>only</b>
    for use in other projects to study the potential benefits of having more crossings.
    Do not ever upload the results of this to OSM.
  </p>

  <button class="btn btn-primary" onclick={enableOps}>I understand</button>
  <button class="btn btn-secondary" onclick={() => (show = false)}>
    Cancel
  </button>
</Modal>
