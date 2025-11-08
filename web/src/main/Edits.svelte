<script lang="ts">
  import { downloadGeneratedFile } from "svelte-utils";
  import { backend, loggedInUser, mutationCounter } from "../";
  import { uploadChangeset } from "osm-api";

  let cmds: any[] = [];
  let idx = 0;

  $: if ($mutationCounter > 0) {
    cmds = $backend ? JSON.parse($backend.getEdits()) : [];
    idx = 0;
  }

  function prev() {
    idx--;
  }

  function next() {
    idx++;
  }

  function clear() {
    $backend!.editClear();
    $mutationCounter++;
  }

  function undo() {
    $backend!.editUndo();
    $mutationCounter++;
  }

  function downloadOsc() {
    downloadGeneratedFile("changes.osc", $backend!.toOsc());
  }

  async function uploadOsc() {
    if (!$loggedInUser) {
      window.alert("You have to log in first");
      return;
    }

    if (cmds.some((cmd) => !("SetTags" in cmd))) {
      window.alert(
        "You've done a bulk operation. You should NOT upload this to OSM -- it's only for testing or usage in you own tool that consumes OSM data.",
      );
      return;
    }

    if (
      !window.confirm(
        "Do you really know what you're doing and want to upload this?",
      )
    ) {
      return;
    }
    let comment = window.prompt("Please describe this changeset");
    if (!comment) {
      return;
    }

    try {
      let id = await uploadChangeset(
        {
          created_by: "Speedwalk",
          comment,
        },
        JSON.parse($backend!.toOsmChangeJson()),
      );
      window.open(`https://www.openstreetmap.org/changeset/${id}`, "_blank");
      // Clear the entire state -- since upstream OSM was just updated, make
      // people re-import the area
      $backend = null;
    } catch (err) {
      window.alert(`Upload failed: ${err}`);
    }
  }
</script>

<div class="card mb-3">
  <div class="card-header">
    <h3>{cmds.length} {cmds.length == 1 ? "edit" : "edits"}</h3>
  </div>
  <div class="card-body">
    {#if cmds.length > 0}
      <div class="mb-1">
        <button class="btn btn-danger" on:click={clear}>Clear edits</button>
        <button
          class="btn btn-danger"
          on:click={undo}
          disabled={cmds.length == 0}
        >
          Undo ({cmds.length})
        </button>
      </div>

      <div class="mb-1">
        <button class="btn btn-secondary" on:click={downloadOsc}>
          Download .osc
        </button>
        <button class="btn btn-secondary" on:click={uploadOsc}>
          Upload changeset
        </button>
      </div>

      <div style="display: flex; justify-content: space-between">
        <button class="btn btn-secondary" on:click={prev} disabled={idx == 0}>
          Previous
        </button>
        <span>{idx + 1} / {cmds.length}</span>
        <button
          class="btn btn-secondary"
          on:click={next}
          disabled={idx == cmds.length - 1}
        >
          Next
        </button>
      </div>

      <p>{JSON.stringify(cmds[idx])}</p>
    {/if}
  </div>
</div>
