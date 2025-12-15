<script lang="ts">
  import * as backendPkg from "../../../backend/pkg";
  import { backend, refreshLoadingScreen } from "../";
  import { overpassQueryForPolygon } from "svelte-utils/overpass";
  import { Loading } from "svelte-utils";

  export let anyEdits: boolean = false;

  let loading = "";

  function describeOsmTimestamp(t: bigint | undefined): string {
    if (t) {
      let d = new Date(1000 * Number(t));
      return `${d.toLocaleString()} (${describeTimeSince(d)} ago)`;
    }
    return "unknown";
  }

  function describeTimeSince(fromDate: Date): string {
    let ms = Date.now() - fromDate.getTime();

    let seconds = Math.floor(ms / 1000);
    let minutes = Math.floor(seconds / 60);
    let hours = Math.floor(minutes / 60);
    let days = Math.floor(hours / 24);

    // Don't be precise
    if (days) {
      return days == 1 ? "1 day" : `${days} days`;
    }
    if (hours) {
      return hours == 1 ? "1 hour" : `${hours} hours`;
    }
    if (minutes) {
      return minutes == 1 ? "1 minute" : `${minutes} minutes`;
    }
    return "a few seconds";
  }

  function clear() {
    if (
      anyEdits &&
      !window.confirm(
        "Changing areas will discard your current edits. Do you want to clear the edits?",
      )
    ) {
      return;
    }
    $backend = null;
  }

  async function refreshData() {
    if (
      anyEdits &&
      !window.confirm(
        "Refreshing OSM data will discard your current edits. Do you want to clear the edits?",
      )
    ) {
      return;
    }

    try {
      loading = "Grabbing new OSM data from Overpass";
      let boundary = JSON.parse($backend!.getBoundary());
      let url = overpassQueryForPolygon(boundary);
      let resp = await fetch(url);
      if (!resp.ok) {
        throw new Error(`Overpass failed: ${resp}`);
      }
      let osmXml = await resp.bytes();

      // TODO Offer to save a copy?
      loading = "Processing Overpass data";
      await refreshLoadingScreen();
      $backend = new backendPkg.Speedwalk(new Uint8Array(osmXml), boundary);
    } catch (err) {
      window.alert(`Couldn't import from Overpass: ${err}`);
    } finally {
      loading = "";
    }
  }
</script>

<Loading {loading} />

<hr class="my-4" />

<div class="mt-4">
  <div class="d-flex gap-2 mb-3">
    <button class="btn btn-secondary" on:click={clear}>
      Load another area
    </button>

    <button class="btn btn-secondary" on:click={refreshData}>
      Refresh OSM data
    </button>
  </div>

  <p>OSM data is from {describeOsmTimestamp($backend?.getOsmTimestamp())}</p>
</div>
