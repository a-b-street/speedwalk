<script lang="ts">
  import { Checkbox, Modal } from "svelte-utils";
  import { backend, mutationCounter, enabledBulkOps } from "../";

  let show = false;
  function enableOps() {
    $enabledBulkOps = true;
    show = false;
  }

  let driveOnLeft = true;
  let onlyMakeSeverances = true;

  function makeAllSidewalksV2() {
    $backend!.editMakeAllSidewalksV2(onlyMakeSeverances);
    $mutationCounter++;
  }

  function connectAllCrossings() {
    $backend!.editConnectAllCrossings();
    $mutationCounter++;
  }

  function assumeTags() {
    $backend!.editAssumeTags(driveOnLeft);
    $mutationCounter++;
  }
</script>

{#if $enabledBulkOps}
  <div class="card">
    <div class="card-header">Bulk operations</div>
    <div class="card-body">
      <div class="card mb-3">
        <div class="card-header">Assume old-style tags on one-ways</div>
        <div class="card-body">
          <Checkbox bind:checked={driveOnLeft}>Drive on the left</Checkbox>
          <button class="btn btn-secondary" on:click={assumeTags}>
            Autoset tags on one-ways
          </button>
        </div>
      </div>

      <div class="card mb-3">
        <div class="card-header">Make all sidewalks</div>
        <div class="card-body">
          <Checkbox bind:checked={onlyMakeSeverances}>
            Only for severances
          </Checkbox>
          <button class="btn btn-secondary" on:click={makeAllSidewalksV2}>
            Make sidewalks
          </button>
        </div>
      </div>

      <button class="btn btn-secondary" on:click={connectAllCrossings}>
        Connect all crossings over severances
      </button>
    </div>
  </div>
{:else}
  <button class="btn btn-secondary" on:click={() => (show = true)}>
    Bulk operations
  </button>
{/if}

<Modal bind:show>
  <h2>Bulk operations</h2>

  <p>
    Speedwalk has some experimental features that can automatically generate
    separate sidewalks on roads tagged with <i>sidewalk = left,right,both</i>
    . This is intended for use in routers and other tools that require separate
    sidewalks. This generation is very error-prone and not meant to ever replace
    mapping sidewalks properly, just as a stop-gap for areas with low coverage.
    Feel free to test it out, but do not ever upload the results to OSM.
  </p>

  <button class="btn btn-primary" on:click={enableOps}>I understand</button>
  <button class="btn btn-secondary" on:click={() => (show = false)}>
    Cancel
  </button>
</Modal>
