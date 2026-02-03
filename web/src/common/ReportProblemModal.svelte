<script lang="ts">
  import type { Map } from "maplibre-gl";
  import { Checkbox, Modal, downloadGeneratedFile } from "svelte-utils";
  import { loggedInUser, map, backend } from "../";
  import { getMapViewport } from "./osmEditorUrls";

  let show = $state(false);
  let submitting = $state(false);
  let success = $state(false);

  export function triggerStart() {
    if (!$map) return;
    start();
  }

  let details = $state("");
  let automaticDetails = $state({});
  let includeUsername = $state(false);
  let includeScreenshot = $state(true);
  let screenshotBlob: Blob | undefined = $state();
  let screenshotURL: string | undefined = $state();

  let filledOut = $derived(details || screenshotURL);

  async function start() {
    show = true;
    automaticDetails = {
      url: window.location.toString(),
      viewport: getViewportHash($map!),
    };
    screenshotBlob = await mapToBlob($map!);
    screenshotURL = URL.createObjectURL(screenshotBlob);
  }

  async function submit() {
    let username = null;
    if (includeUsername && $loggedInUser) {
      username = $loggedInUser.name;
    }

    let screenshot = null;
    if (includeScreenshot && screenshotBlob) {
      screenshot = await blobToBase64(screenshotBlob);
    }

    let req = {
      ...automaticDetails,
      details,
      screenshot,
      username,
    };
    try {
      submitting = true;
      let resp = await fetch(
        "https://walknet.abstreet.uk/problems/speedwalk_problems",
        {
          method: "POST",
          body: JSON.stringify(req),
          headers: {
            "Content-Type": "application/json",
          },
        },
      );
      if (!resp.ok) {
        throw new Error(`Bad response: ${resp.status}`);
      }
    } catch (err) {
      window.alert(`Failed to report a problem: ${err}`);
      submitting = false;
      return;
    }

    submitting = false;
    cancel();
    success = true;
  }

  function cancel() {
    show = false;
    details = "";
    automaticDetails = {};
    screenshotBlob = undefined;
    // Don't reset email
    if (screenshotURL) {
      URL.revokeObjectURL(screenshotURL);
      screenshotURL = undefined;
    }
  }

  function mapToBlob(map: Map): Promise<Blob> {
    return new Promise((resolve, reject) => {
      map.redraw();
      map.once("idle", () => {
        map.getCanvas().toBlob((blob) => {
          if (blob) {
            resolve(blob);
          } else {
            reject("no blob");
          }
        });
      });
    });
  }

  function blobToBase64(blob: Blob): Promise<string> {
    return new Promise((resolve, reject) => {
      let reader = new FileReader();
      reader.readAsDataURL(blob);
      reader.onloadend = function () {
        if (reader.result) {
          resolve(reader.result as string);
        } else {
          reject("base64 failed");
        }
      };
    });
  }

  // Calculate precision for coordinates based on zoom level
  // derived from equation: 512px * 2^z / 360 / 10^d < 0.5px
  function calculatePrecision(zoom: number): number {
    return Math.ceil((zoom * Math.LN2 + Math.log(512 / 360 / 0.5)) / Math.LN10);
  }

  function roundCoordinate(value: number, zoom: number): number {
    const precision = calculatePrecision(zoom);
    const m = Math.pow(10, precision);
    return Math.round(value * m) / m;
  }

  // TODO Double hash
  // Adapted from https://github.com/maplibre/maplibre-gl-js/blob/5d7e6d52000a8569ac2308a9aef14c98933eb0d8/src/ui/hash.ts
  function getViewportHash(map: Map): string {
    const viewport = getMapViewport(map);
    if (!viewport) return "";

    const zoom = Math.round(viewport.zoom * 100) / 100;
    const lat = roundCoordinate(viewport.lat, zoom);
    const lng = roundCoordinate(viewport.lng, zoom);
    let hash = `${zoom}/${lat}/${lng}`;

    const bearing = map.getBearing();
    const pitch = map.getPitch();
    if (bearing || pitch) {
      hash += `/${Math.round(bearing * 10) / 10}`;
    }
    if (pitch) {
      hash += `/${Math.round(pitch)}`;
    }
    return `#${hash}`;
  }

  function formatViewportForFilename(map: Map): string {
    const viewport = getMapViewport(map);
    if (!viewport) return "";

    const zoom = Math.round(viewport.zoom * 100) / 100;
    const lat = roundCoordinate(viewport.lat, zoom);
    const lng = roundCoordinate(viewport.lng, zoom);
    return `${zoom}_${lat}_${lng}`;
  }

  async function exportDebugOsm() {
    if (!$map || !$backend) return;

    try {
      // Get map bounds
      const bounds = $map.getBounds();
      const bbox = new Float64Array([
        bounds.getWest(),
        bounds.getSouth(),
        bounds.getEast(),
        bounds.getNorth(),
      ]);

      // Export OSM data
      const osmXml = $backend.exportOsmForViewport(bbox);

      // Generate filename with zoom, lat, lng
      const viewportStr = formatViewportForFilename($map);
      const filename = `speedwalk_debug_${viewportStr}.osm.xml`;

      // Download file
      downloadGeneratedFile(filename, osmXml);
    } catch (err) {
      window.alert(`Failed to export debug OSM data: ${err}`);
    }
  }
</script>

<Modal bind:show={success}>
  <h2>Report a problem</h2>

  <p>
    Done, thank you for submitting.
    {#if includeUsername && $loggedInUser}
      You may be contacted later when this problem has been fixed.
    {/if}
  </p>
  <button class="btn btn-primary" onclick={() => (success = false)}>OK</button>
</Modal>

<Modal bind:show closeable={false}>
  <div class="d-flex justify-content-between">
    <h2>Report a problem</h2>
    <button class="btn-close" onclick={cancel} aria-label="Close"></button>
  </div>

  <p>You can report a problem with Speedwalk.</p>

  <fieldset class="mb-3">
    <legend>
      Please describe the problem
      <br />
      (if it's not obvious from the screenshot below)
    </legend>
    <textarea
      class="form-control"
      rows="4"
      placeholder="Details"
      bind:value={details}
    ></textarea>
  </fieldset>

  <Checkbox bind:checked={includeScreenshot}>Include screenshot</Checkbox>

  {#if $loggedInUser}
    <Checkbox bind:checked={includeUsername}>
      Include your OSM username ({$loggedInUser.name}) in the report and be
      contacted when the problem is fixed
    </Checkbox>
  {/if}

  {#if includeScreenshot}
    <div class="img-container mb-3">
      {#if screenshotURL}
        <img src={screenshotURL} alt="Screenshot" />
      {:else}
        <p>
          Loading screenshot... (sometimes you have to cancel and try again)
        </p>
      {/if}
    </div>
  {/if}

  <div class="d-flex">
    {#if submitting}
      <button class="btn btn-primary me-3" disabled>
        <div class="spinner-border" role="status">
          <span class="visually-hidden">Loading...</span>
        </div>
        Submit report
      </button>
    {:else}
      <button
        class="btn btn-primary me-3"
        onclick={submit}
        disabled={!filledOut}
      >
        Submit report
      </button>
    {/if}

    {#if $backend}
      <button class="btn btn-secondary me-3" onclick={exportDebugOsm}>
        Export debug OSM data
      </button>
    {/if}

    <button class="btn btn-secondary" onclick={cancel}>Cancel</button>
  </div>
</Modal>

<style>
  .img-container {
    width: 600px;
    height: 300px;
    border: 1px solid black;
    display: flex;
    align-items: center;
    justify-content: center;
    background: grey;
  }

  img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
  }
</style>
