<script lang="ts">
  import BasemapSelector from "./BasemapSelector.svelte";
  import EditorDropdown from "./EditorDropdown.svelte";
  import ReportProblemButton from "./ReportProblemButton.svelte";

  export let basemap: string;
  export let bottom = "60px";
</script>

<div class="action-bar" style:bottom>
  <BasemapSelector bind:basemap />
  <EditorDropdown />
  <ReportProblemButton />
</div>

<style>
  .action-bar {
    position: absolute;
    left: 10px;
    background: white;
    padding: 4px;
    display: flex;
    gap: 4px;
    align-items: center;
  }
</style>
