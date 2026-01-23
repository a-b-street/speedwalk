<script lang="ts">
  import * as backendPkg from "../../../backend/pkg";
  import { backend, refreshLoadingScreen, anyEdits } from "../";
  import {
    overpassQueryForPolygon,
    fetchOverpass,
    OverpassServerSelector,
  } from "svelte-utils/osm";
  import {
    downloadGeneratedFile,
    Checkbox,
    Loading,
    Modal,
  } from "svelte-utils";

  let show = $state(false);
  let loading = $state("");
  let saveCopy = $state(false);

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
      $anyEdits &&
      !window.confirm(
        "Changing areas will discard your current edits. Do you want to clear the edits?",
      )
    ) {
      return;
    }
    $backend = null;
    $anyEdits = false;
  }

  async function refreshData() {
    if (
      $anyEdits &&
      !window.confirm(
        "Refreshing OSM data will discard your current edits. Do you want to clear the edits?",
      )
    ) {
      return;
    }

    // Loading should cover up the Modal, but it doesn't
    show = false;

    try {
      loading = "Grabbing new OSM data from Overpass";
      let boundary = JSON.parse($backend!.getBoundary());
      let resp = await fetchOverpass(overpassQueryForPolygon(boundary));
      let osmXml = await resp.bytes();

      if (saveCopy) {
        let text = new TextDecoder().decode(osmXml);
        downloadGeneratedFile("refreshed_import.osm.xml", text);
      }

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

<Modal bind:show>
  <h1>Choose an area to work on</h1>

  <div class="mb-3">
    <button class="btn btn-secondary" onclick={clear}>Import a new area</button>
  </div>

  <div class="card card-body mb-3">
    <p>OSM data is from {describeOsmTimestamp($backend?.getOsmTimestamp())}</p>

    <button class="btn btn-secondary" onclick={refreshData}>
      Refresh OSM data
    </button>

    <Checkbox bind:checked={saveCopy}>
      Save a copy of the latest osm.xml after refreshing
    </Checkbox>

    <OverpassServerSelector />
  </div>

  <button class="btn btn-primary" onclick={() => (show = false)}>Cancel</button>
</Modal>

<button class="btn btn-outline-secondary" onclick={() => (show = true)}>
  &larr; Load another area
</button>
