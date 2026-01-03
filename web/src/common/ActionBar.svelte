<script lang="ts">
  import BasemapSelector from "./BasemapSelector.svelte";
  import EditorDropdown from "./EditorDropdown.svelte";
  import ReportProblemButton from "./ReportProblemButton.svelte";
  import Mapillary from "./Mapillary.svelte";

  let { basemap = $bindable() }: { basemap: string } = $props();

  let bottom = "60px";
</script>

<div class="action-bar" style:bottom>
  <BasemapSelector bind:basemap />
  <EditorDropdown />
  <ReportProblemButton />
  <Mapillary />
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
    z-index: 10;
  }
</style>
