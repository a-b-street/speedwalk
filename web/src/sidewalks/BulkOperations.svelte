<script lang="ts">
  import CollapsibleCard from "../common/CollapsibleCard.svelte";
  import { Checkbox, Modal, Loading } from "svelte-utils";
  import {
    backend,
    mutationCounter,
    enabledBulkOps,
    refreshLoadingScreen,
  } from "../";

  let show = $state(false);
  function enableOps() {
    $enabledBulkOps = true;
    show = false;
  }

  let loading = $state("");
  let driveOnLeft = $state(true);
  let onlyMakeSeverances = $state(true);

  async function makeAllSidewalks() {
    loading = "Generating sidewalks";
    await refreshLoadingScreen();
    try {
      $backend!.editMakeAllSidewalks(onlyMakeSeverances);
      $mutationCounter++;
    } catch (err) {
      window.alert(`Error: ${err}`);
    } finally {
      loading = "";
    }
  }

  async function connectAllCrossings() {
    loading = "Connecting crossings";
    await refreshLoadingScreen();
    try {
      $backend!.editConnectAllCrossings();
      $mutationCounter++;
    } catch (err) {
      window.alert(`Error: ${err}`);
    } finally {
      loading = "";
    }
  }

  async function assumeTags() {
    loading = "Inferring sidewalks around one-ways";
    await refreshLoadingScreen();
    try {
      $backend!.editAssumeTags(driveOnLeft);
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
      <div class="card mb-3">
        <div class="card-header">Assume old-style tags on one-ways</div>
        <div class="card-body">
          <Checkbox bind:checked={driveOnLeft}>Drive on the left</Checkbox>
          <button class="btn btn-secondary" onclick={assumeTags}>
            Autoset tags on one-ways
          </button>
        </div>
      </div>

      <div class="card mb-3">
        <div class="card-header">Make all sidewalks</div>
        <div class="card-body">
          <Checkbox bind:checked={onlyMakeSeverances}>
            Only for major roads
          </Checkbox>
          <button class="btn btn-secondary" onclick={makeAllSidewalks}>
            Make sidewalks
          </button>
        </div>
      </div>

      <button class="btn btn-secondary" onclick={connectAllCrossings}>
        Connect all crossing nodes
      </button>
    {/snippet}
  </CollapsibleCard>
{:else}
  <button class="btn btn-secondary" onclick={() => (show = true)}>
    Bulk operations
  </button>
{/if}

<Modal bind:show>
  <h2>Bulk operations</h2>

  <p>
    Speedwalk has some experimental features that can automatically generate
    separate sidewalks on roads tagged with <i>sidewalk = left,right,both</i>
    . This is intended for use in routers and other tools that require separate sidewalks.
    This generation is very error-prone and not meant to ever replace mapping sidewalks
    properly, just as a stop-gap for areas with low coverage. Feel free to test it
    out, but do not ever upload the results to OSM.
  </p>

  <button class="btn btn-primary" onclick={enableOps}>I understand</button>
  <button class="btn btn-secondary" onclick={() => (show = false)}>
    Cancel
  </button>
</Modal>
