<script lang="ts">
  import { Checkbox, Loading, LocalStorageWrapper } from "svelte-utils";
  import {
    backend,
    mutationCounter,
    refreshLoadingScreen,
    onlyMajorRoadsBulk,
    includeCrossingNoBulk,
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

  let loading = $state("");
  let driveOnLeft = $state(true);

  async function generateCrossingsMajor() {
    loading = "Generating crossings on major roads";
    await refreshLoadingScreen();
    try {
      $backend!.editGenerateMissingCrossings({
        ...defaultCrossingOptions,
        only_major_roads: true,
      });
      $mutationCounter++;
    } catch (err) {
      window.alert(`Error: ${err}`);
    } finally {
      loading = "";
    }
  }

  async function generateCrossingsMinor() {
    loading = "Generating crossings on minor roads";
    await refreshLoadingScreen();
    try {
      $backend!.editGenerateMissingCrossings({
        ...defaultCrossingOptions,
        only_major_roads: false,
        ignore_utility_roads: true,
      });
      $mutationCounter++;
    } catch (err) {
      window.alert(`Error: ${err}`);
    } finally {
      loading = "";
    }
  }

  async function generateCrossingsAll() {
    loading = "Generating missing crossings";
    await refreshLoadingScreen();
    try {
      $backend!.editGenerateMissingCrossings(defaultCrossingOptions);
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
  <div class="card-header">Generate crossings</div>
  <div class="card-body">
    <button
      class="btn btn-secondary me-2 mb-2"
      onclick={generateCrossingsMajor}
    >
      Generate crossings on major roads
    </button>
    <button
      class="btn btn-secondary me-2 mb-2"
      onclick={generateCrossingsMinor}
    >
      Generate crossings on minor roads (excluding service)
    </button>
    <button class="btn btn-secondary mb-2" onclick={generateCrossingsAll}>
      Generate imaginary crossings where they're missing
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
