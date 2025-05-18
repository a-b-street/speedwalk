<script lang="ts">
  import { backend, mutationCounter } from "./";

  let cmds: any[] = [];

  $: if ($mutationCounter > 0) {
    cmds = JSON.parse($backend!.getEdits());
  }

  function clear() {
    $backend!.editClear();
    $mutationCounter++;
  }
</script>

<h3>{cmds.length} {cmds.length == 1 ? "edit" : "edits"}</h3>
{#if cmds.length > 0}
  <button on:click={clear}>Clear edits</button>
{/if}
<p>{JSON.stringify(cmds)}</p>
