<script lang="ts">
  import { basemapStyles } from "../";
  import { basemapStyles as originalBasemapStyles } from "svelte-utils/map";
  import AddBasemap from "./AddBasemap.svelte";

  export let basemap: string;

  let showBasemapAddBasemapModal = false;

  $: defaultBasemaps = Object.keys(originalBasemapStyles);
  $: allBasemaps = Object.keys($basemapStyles);
  $: customBasemaps = allBasemaps.filter((name) => !defaultBasemaps.includes(name));
  $: displayName = basemap || allBasemaps[0] || "";
</script>

<div class="basemap-selector">
  <div class="btn-group">
    <button
      class="btn btn-outline-secondary dropdown-toggle basemap-button"
      type="button"
      data-bs-toggle="dropdown"
      aria-expanded="false"
    >
      {displayName}
    </button>
    <ul class="dropdown-menu">
      {#each defaultBasemaps as name}
        <li>
          <button
            class="dropdown-item"
            class:active={basemap === name}
            type="button"
            on:click={() => (basemap = name)}
          >
            {name}
          </button>
        </li>
      {/each}
      {#if customBasemaps.length > 0}
        <li><hr class="dropdown-divider" /></li>
        {#each customBasemaps as name}
          <li>
            <button
              class="dropdown-item"
              class:active={basemap === name}
              type="button"
              on:click={() => (basemap = name)}
            >
              {name}
            </button>
          </li>
        {/each}
      {/if}
      <li><hr class="dropdown-divider" /></li>
      <li>
        <button
          class="dropdown-item"
          type="button"
          on:click={() => (showBasemapAddBasemapModal = true)}
        >
          <i class="fa-solid fa-square-plus me-2"></i>
          Add custom basemap
        </button>
      </li>
    </ul>
  </div>
</div>

<AddBasemap bind:show={showBasemapAddBasemapModal} />

<style>
  .basemap-selector {
    display: flex;
    align-items: center;
  }

  .basemap-button {
    width: 50%;
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .basemap-button::after {
    float: right;
    margin-top: 0.5em;
  }

  :global(.basemap-selector .dropdown-menu) {
    max-height: 300px;
    overflow-y: auto;
    padding: 0.25rem 0;
  }

  :global(.basemap-selector .dropdown-item) {
    padding: 0.25rem 0.75rem;
    white-space: nowrap;
    margin: 0;
  }

  :global(.basemap-selector .dropdown-menu li) {
    margin: 0;
  }
</style>
