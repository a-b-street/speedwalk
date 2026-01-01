<script lang="ts">
  import { map } from "../";
  import {
    getMapViewport,
    getOsmUrl,
    getIdUrl,
    getKiwidUrl,
    getRapidUrl,
    getJosmUrlFromMap,
    getMapillaryUrl,
  } from "./osmEditorUrls";

  const options = ["OSM", "iD", "Kiwid", "Rapid", "JOSM", "Mapillary"];

  // Instead of updating hrefs constantly as the viewport changes, lazily figure out the URL
  function getUrl(label: string): string | undefined {
    const viewport = getMapViewport($map);
    if (!$map || !viewport) {
      return undefined;
    }
    return {
      OSM: getOsmUrl(viewport.zoom, viewport.lat, viewport.lng),
      iD: getIdUrl(viewport.zoom, viewport.lat, viewport.lng),
      Kiwid: getKiwidUrl(viewport.zoom, viewport.lat, viewport.lng),
      Rapid: getRapidUrl(viewport.zoom, viewport.lat, viewport.lng),
      JOSM: getJosmUrlFromMap($map),
      Mapillary: getMapillaryUrl(viewport.zoom, viewport.lat, viewport.lng),
    }[label];
  }
</script>

<div class="btn-group">
  <button
    class="btn btn-outline-secondary dropdown-toggle"
    type="button"
    data-bs-toggle="dropdown"
    aria-expanded="false"
    title="Open in OSM editors"
  >
    <i class="fa-solid fa-pen"></i>
  </button>
  <ul class="dropdown-menu">
    {#each options as label}
      <li>
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          class="dropdown-item"
          href="#"
          onclick={(e) => {
            e.preventDefault();
            window.open(getUrl(label), "_blank", "noopener,noreferrer");
          }}
        >
          {label}
        </a>
      </li>
    {/each}
  </ul>
</div>
