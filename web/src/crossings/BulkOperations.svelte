<script lang="ts">
  import CollapsibleCard from "../common/CollapsibleCard.svelte";
  import { Modal, Loading } from "svelte-utils";
  import {
    backend,
    mutationCounter,
    enabledBulkOps,
    refreshLoadingScreen,
  } from "../";

  export let options: any;

  let show = false;
  function enableOps() {
    $enabledBulkOps = true;
    show = false;
  }

  let loading = "";

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
    <div slot="header">Bulk operations</div>
    <div slot="body">
      <button class="btn btn-secondary" on:click={generateCrossings}>
        Generate imaginary crossings where they're missing
      </button>
    </div>
  </CollapsibleCard>
{:else}
  <button class="btn btn-secondary" on:click={() => (show = true)}>
    Bulk operations
  </button>
{/if}

<Modal bind:show>
  <h2>Bulk operations</h2>

  <p>
    Speedwalk has some experimental features that can automatically generate
    crossing nodes on every arm of a junction where they're expected. This is
    intended <b>only</b>
    for use in other projects to study the potential benefits of having more crossings.
    Do not ever upload the results of this to OSM.
  </p>

  <button class="btn btn-primary" on:click={enableOps}>I understand</button>
  <button class="btn btn-secondary" on:click={() => (show = false)}>
    Cancel
  </button>
</Modal>
