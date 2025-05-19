<script lang="ts">
  import { downloadGeneratedFile } from "svelte-utils";
  import { backend, mutationCounter } from "./";

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
</script>

<h3>{cmds.length} {cmds.length == 1 ? "edit" : "edits"}</h3>
{#if cmds.length > 0}
  <button on:click={clear}>Clear edits</button>
  <button on:click={downloadOsc}>Download .osc</button>
{/if}
<p>{JSON.stringify(cmds)}</p>
