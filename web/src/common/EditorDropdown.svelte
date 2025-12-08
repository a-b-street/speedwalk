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

  function getEditorUrls() {
    if (!$map) return null;
    const viewport = getMapViewport($map);
    if (!viewport) return null;
    return [
      {
        label: "OSM",
        url: getOsmUrl(viewport.zoom, viewport.lat, viewport.lng),
      },
      { label: "iD", url: getIdUrl(viewport.zoom, viewport.lat, viewport.lng) },
      {
        label: "Kiwid",
        url: getKiwidUrl(viewport.zoom, viewport.lat, viewport.lng),
      },
      {
        label: "Rapid",
        url: getRapidUrl(viewport.zoom, viewport.lat, viewport.lng),
      },
      { label: "JOSM", url: getJosmUrlFromMap($map) },
      {
        label: "Mapillary",
        url: getMapillaryUrl(viewport.zoom, viewport.lat, viewport.lng),
      },
    ];
  }
</script>

{#if $map}
  {@const urls = getEditorUrls()}
  {#if urls}
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
        {#each urls as { label, url }}
          <li>
            <a
              class="dropdown-item"
              href={url}
              target="_blank"
              rel="noopener noreferrer"
            >
              {label}
            </a>
          </li>
        {/each}
      </ul>
    </div>
  {/if}
{/if}
