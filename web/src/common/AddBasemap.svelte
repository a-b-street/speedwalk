<script lang="ts">
  import type { StyleSpecification } from "maplibre-gl";
  import { basemapStyles } from "svelte-utils/map";
  import { Modal } from "svelte-utils";

  let { show = $bindable() }: { show: boolean } = $props();

  let localStorageKey = "speedwalk-custom-basemaps";

  interface CustomBasemap {
    name: string;
    tileURL: string;
    attribution: string;
  }

  // Setup from local storage. Don't attempt any validation.
  let customBasemaps: CustomBasemap[] = $state(
    JSON.parse(window.localStorage.getItem(localStorageKey) || "[]"),
  );
  // svelte-ignore state_referenced_locally
  for (let x of customBasemaps) {
    basemapStyles.set(x.name, makeStyle(x.tileURL, x.attribution));
  }

  let name = $state("");
  let tileURL = $state("");
  let attribution = $state("");

  function makeStyle(tileURL: string, attribution: string): StyleSpecification {
    let maptilerKey = "MZEJTanw3WpxRvt7qDfo";
    let glyphs = `https://api.maptiler.com/fonts/{fontstack}/{range}.pbf?key=${maptilerKey}`;
    return {
      version: 8,
      sources: {
        "raster-tiles": {
          type: "raster",
          tiles: [tileURL],
          tileSize: 256,
          attribution,
        },
      },
      layers: [
        {
          id: "raster-basemap",
          type: "raster",
          source: "raster-tiles",
        },
        // TODO Other layers z-order underneath this layer, which has to exist.
        // Figure out a more dynamic z-ordering approach.
        {
          id: "Road labels",
          type: "background",
          layout: {
            visibility: "none",
          },
        },
      ],
      glyphs,
    };
  }

  function addBasemap() {
    customBasemaps.push({ name, tileURL, attribution });
    window.localStorage.setItem(
      localStorageKey,
      JSON.stringify(customBasemaps),
    );
    basemapStyles.set(name, makeStyle(tileURL, attribution));

    name = "";
    tileURL = "";
    attribution = "";
  }

  function removeBasemap(name: string) {
    customBasemaps = customBasemaps.filter((x) => x.name != name);
    window.localStorage.setItem(
      localStorageKey,
      JSON.stringify(customBasemaps),
    );
    basemapStyles.delete(name);
    // TODO If the current basemap was set to this, it'll last till the user changes it
  }
</script>

<Modal bind:show>
  <h1>Customize basemaps</h1>

  {#each customBasemaps as x}
    <div>
      <button class="btn btn-danger mb-3" onclick={() => removeBasemap(x.name)}>
        Remove {x.name}
      </button>
    </div>
  {/each}

  <div class="card">
    <div class="card-header">Add a basemap</div>
    <div class="card-body">
      <p>
        You can add custom basemaps if you know a tile URL and that the imagery
        has a license allowing it to be used for editing OSM.
      </p>
      <ul>
        <li>
          <a
            href="https://github.com/osmlab/editor-layer-index"
            target="_blank"
          >
            editor-layer-index
          </a>
          is a great reference, or you could check in iD or JOSM and figure out which
          one you like to use for your area.
        </li>
        <li>
          If you copy a URL from there, you may need to manually substitute some
          parts of the URL to make it work.
        </li>
        <li>
          <b>&lbrace;zoom&rbrace;</b>
          &rarr;
          <b>&lbrace;z&rbrace;</b>
        </li>
        <li>
          <b>&lbrace;proj&rbrace;</b>
          &rarr;
          <b>&lbrace;EPSG:3857&rbrace;</b>
        </li>
        <li>
          <b>&lbrace;width&rbrace;</b>
          &rarr;
          <b>256</b>
        </li>
        <li>
          <b>&lbrace;height&rbrace;</b>
          &rarr;
          <b>256</b>
        </li>
        <li>
          <b>&lbrace;bbox&rbrace;</b>
          &rarr;
          <b>&lbrace;bbox-epsg-3857&rbrace;</b>
        </li>
        <li>Some sources have CORS restrictions and won't work</li>
      </ul>

      <div>
        <label class="form-label">
          Name:
          <input class="form-control" type="text" bind:value={name} />
        </label>
      </div>
      <div>
        <label class="form-label">
          Tile URL:
          <textarea
            class="form-control"
            rows="3"
            cols="80"
            bind:value={tileURL}
          ></textarea>
        </label>
      </div>
      <div>
        <label class="form-label">
          Attribution:
          <input class="form-control" type="text" bind:value={attribution} />
        </label>
      </div>
      <button
        class="btn btn-primary"
        disabled={!name || !tileURL || !attribution}
        onclick={addBasemap}
      >
        Add basemap
      </button>
    </div>
  </div>

  <div class="mt-5">
    <button class="btn btn-primary" onclick={() => (show = false)}>Done</button>
  </div>
</Modal>
