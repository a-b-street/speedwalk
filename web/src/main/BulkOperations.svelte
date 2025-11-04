<script lang="ts">
  import { Checkbox } from "svelte-utils";
  import { backend, mutationCounter } from "../";

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
