<script lang="ts">
  import type { Map } from "maplibre-gl";
  import { Checkbox } from "svelte-utils";
  import { loggedInUser, map } from "./";
  import Modal from "./Modal.svelte";

  let show = false;
  let submitting = false;
  let success = false;

  let details = "";
  let automaticDetails = {};
  let includeUsername = false;
  let includeScreenshot = true;
  let screenshotBlob: Blob | undefined = undefined;
  let screenshotURL: string | undefined = undefined;

  $: filledOut = details || screenshotURL;

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

  // TODO Double hash
  // Adapted from https://github.com/maplibre/maplibre-gl-js/blob/5d7e6d52000a8569ac2308a9aef14c98933eb0d8/src/ui/hash.ts
  function getViewportHash(map: Map): string {
    let center = map.getCenter();
    let zoom = Math.round(map.getZoom() * 100) / 100;
    // derived from equation: 512px * 2^z / 360 / 10^d < 0.5px
    let precision = Math.ceil(
      (zoom * Math.LN2 + Math.log(512 / 360 / 0.5)) / Math.LN10,
    );
    let m = Math.pow(10, precision);
    let lat = Math.round(center.lat * m) / m;
    let lng = Math.round(center.lng * m) / m;
    let hash = `${zoom}/${lat}/${lng}`;

    let bearing = map.getBearing();
    let pitch = map.getPitch();
    if (bearing || pitch) {
      hash += `/${Math.round(bearing * 10) / 10}`;
    }
    if (pitch) {
      hash += `/${Math.round(pitch)}`;
    }
    return `#${hash}`;
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
  <button class="btn btn-primary" on:click={() => (success = false)}>OK</button>
</Modal>

<Modal bind:show closeable={false}>
  <div class="d-flex justify-content-between">
    <h2>Report a problem</h2>
    <button class="btn-close" on:click={cancel} aria-label="Close"></button>
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
        on:click={submit}
        disabled={!filledOut}
      >
        Submit report
      </button>
    {/if}

    <button class="btn btn-secondary" on:click={cancel}>Cancel</button>
  </div>
</Modal>

<div class="launcher">
  <button class="btn btn-secondary" on:click={start} disabled={show}>
    <i class="fa-solid fa-triangle-exclamation"></i>
    Report problem
  </button>
</div>

<style>
  .launcher {
    z-index: 100;
    position: absolute;
    left: 10px;
    bottom: 140px;
  }

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
