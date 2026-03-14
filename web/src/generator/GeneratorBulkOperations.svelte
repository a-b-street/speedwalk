<script lang="ts">
  import { Checkbox, Loading, LocalStorageWrapper } from "svelte-utils";
  import {
    backend,
    mutationCounter,
    refreshLoadingScreen,
    onlyMajorRoadsBulk,
    includeCrossingNoBulk,
    crossingScopeBulk,
    type CrossingScopeBulk,
  } from "../";

  const defaultCrossingOptions = {
    only_major_roads: true,
    ignore_utility_roads: true,
    ignore_cycleways: true,
    ignore_footways: true,
    ignore_roundabouts: true,
    ignore_motorways: true,
    max_distance: 40,
  };

  const crossingScopeOptions: { value: CrossingScopeBulk; label: string }[] = [
    { value: "major", label: "Major roads only" },
    { value: "minor", label: "Major + minor, excl. service/track" },
    {
      value: "all",
      label: "All roads (excl. cycleways, footways, motorways, roundabouts)",
    },
  ];

  let loading = $state("");
  let driveOnLeft = $state(true);

  async function generateCrossings() {
    const scope = $crossingScopeBulk;
    const options = {
      ...defaultCrossingOptions,
      only_major_roads: scope === "major",
      ignore_utility_roads: scope !== "all",
    };
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

  async function makeAllSidewalks() {
    loading = "Generating sidewalks";
    await refreshLoadingScreen();
    try {
      $backend!.editMakeAllSidewalks($onlyMajorRoadsBulk);
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
      $backend!.editConnectAllCrossings($includeCrossingNoBulk);
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

<div class="card mb-3">
  <div class="card-header">
    <LocalStorageWrapper>
      <span>Generate crossings</span>
    </LocalStorageWrapper>
  </div>
  <div class="card-body">
    <div class="mb-2">
      {#each crossingScopeOptions as opt}
        <div class="form-check">
          <input
            type="radio"
            class="form-check-input"
            name="crossingScope"
            id="crossingScope-{opt.value}"
            value={opt.value}
            bind:group={$crossingScopeBulk}
          />
          <label class="form-check-label" for="crossingScope-{opt.value}">
            {opt.label}
          </label>
        </div>
      {/each}
    </div>
    <button class="btn btn-secondary" onclick={generateCrossings}>
      Generate missing crossings
    </button>
  </div>
</div>

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
    <LocalStorageWrapper>
      <Checkbox bind:checked={$onlyMajorRoadsBulk}>
        Only for major roads
      </Checkbox>
    </LocalStorageWrapper>
    <button class="btn btn-secondary" onclick={makeAllSidewalks}>
      Make sidewalks
    </button>
  </div>
</div>

<div class="card mb-3">
  <div class="card-header">Connect all crossing nodes</div>
  <div class="card-body">
    <LocalStorageWrapper>
      <Checkbox bind:checked={$includeCrossingNoBulk}>
        Include crossing=no
      </Checkbox>
    </LocalStorageWrapper>
    <button class="btn btn-secondary" onclick={connectAllCrossings}>
      Create a way for every crossing node
    </button>
  </div>
</div>
