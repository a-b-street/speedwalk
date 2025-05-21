<script lang="ts">
  import Auth from "./Auth.svelte";
  import { downloadGeneratedFile } from "svelte-utils";
  import { backend, mutationCounter } from "./";
  import { previewSidewalk } from "./sidewalks/";
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

  function downloadOsc() {
    downloadGeneratedFile("changes.osc", $backend!.toOsc());
  }

  async function uploadOsc() {
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
      $previewSidewalk = null;
    } catch (err) {
      window.alert(`Upload failed: ${err}`);
    }
  }
</script>

<Auth />

<h3>{cmds.length} {cmds.length == 1 ? "edit" : "edits"}</h3>
{#if cmds.length > 0}
  <button class="outline" on:click={clear}>Clear edits</button>
  <button class="outline" on:click={downloadOsc}>Download .osc</button>
  <button on:click={uploadOsc}>Upload changeset</button>

  <div style="display: flex; justify-content: space-between">
    <button class="outline" on:click={prev} disabled={idx == 0}>
      Previous
    </button>
    <span>{idx + 1} / {cmds.length}</span>
    <button class="outline" on:click={next} disabled={idx == cmds.length - 1}>
      Next
    </button>
  </div>

  <p>{JSON.stringify(cmds[idx])}</p>
{/if}
