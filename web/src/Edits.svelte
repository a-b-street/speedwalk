<script lang="ts">
  import Auth from "./Auth.svelte";
  import { downloadGeneratedFile } from "svelte-utils";
  import { backend, mutationCounter } from "./";
  import { uploadChangeset } from "osm-api";

  let cmds: any[] = [];

  $: if ($mutationCounter > 0) {
    cmds = JSON.parse($backend!.getEdits());
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
      clear();
    } catch (err) {
      window.alert(`Upload failed: ${err}`);
    }
  }
</script>

<Auth />

<h3>{cmds.length} {cmds.length == 1 ? "edit" : "edits"}</h3>
{#if cmds.length > 0}
  <button on:click={clear}>Clear edits</button>
  <button on:click={downloadOsc}>Download .osc</button>
  <button on:click={uploadOsc}>Upload changeset</button>
{/if}
<p>{JSON.stringify(cmds)}</p>
