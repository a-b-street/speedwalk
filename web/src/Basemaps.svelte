<script lang="ts">
  import { basemapStyles } from "./";
  import AddBasemap from "./AddBasemap.svelte";

  export let basemap: string;
  export let bottom = "60px";

  let show = false;
</script>

<div style:bottom>
  <!-- svelte-ignore a11y-missing-content -->
  <h5 class="fa-solid fa-layer-group me-2 align-self-center"></h5>
  <button class="btn btn-outline-secondary" on:click={() => (show = true)}>
    <i class="fa-solid fa-square-plus"></i>
  </button>
  <select class="form-select" bind:value={basemap}>
    {#each Object.keys($basemapStyles) as value}
      <option {value}>{value}</option>
    {/each}
  </select>
</div>

<AddBasemap bind:show />

<style>
  div {
    position: absolute;
    left: 10px;

    background: white;
    padding: 4px;

    display: flex;
  }
</style>
