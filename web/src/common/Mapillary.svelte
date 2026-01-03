<script lang="ts">
  import {
    hoverStateFilter,
    VectorTileSource,
    LineLayer,
    CircleLayer,
  } from "svelte-maplibre";
  import { Checkbox } from "svelte-utils";
  import CollapsibleCard from "./CollapsibleCard.svelte";

  let show = $state(false);
  let pinImage: number | null = $state(null);

  // Making this public means anybody could burn through our quota. Embedding in git means it's
  // easier to scrape, but in any case, anybody using the app can just see the key in network
  // requests. Ideally Mapillary would restrict the domains this key can be used from. If we really
  // cared, we could proxy all requests through a server that hides the key. But it's free tier and
  // read-only access, so really doesn't matter much.
  let accessToken = "MLY|25548189401504437|3c9576aa434c09888b5de1c28f13b1df";
</script>

<Checkbox bind:checked={show}>Mapillary</Checkbox>

<VectorTileSource
  tiles={[
    `https://tiles.mapillary.com/maps/vtp/mly1_public/2/{z}/{x}/{y}?access_token=${accessToken}`,
  ]}
  maxzoom={14}
>
  <LineLayer
    sourceLayer="sequence"
    paint={{
      "line-width": 2,
      "line-color": "green",
    }}
    layout={{
      visibility: show ? "visible" : "none",
    }}
  />

  <CircleLayer
    sourceLayer="image"
    manageHoverState
    paint={{
      "circle-radius": 10,
      "circle-color": hoverStateFilter("green", "orange"),
      "circle-stroke-color": "black",
      "circle-stroke-width": 1,
    }}
    layout={{
      visibility: show ? "visible" : "none",
    }}
    hoverCursor="pointer"
    onclick={(e) => (pinImage = e.features[0].properties!.id)}
  />
</VectorTileSource>

<div class="viewer-container" style:visibility={show ? "visible" : "hidden"}>
  <CollapsibleCard>
    {#snippet header()}Mapillary{/snippet}
    {#snippet body()}
      {#if pinImage}
        <iframe
          title="Mapillary"
          src={`https://www.mapillary.com/embed?image_key=${pinImage}&style=photo`}
          height="240"
          width="320"
          frameborder="0"
        ></iframe>
      {/if}
    {/snippet}
  </CollapsibleCard>
</div>

<style>
  .viewer-container {
    z-index: 100;
    position: absolute;
    left: 10px;
    bottom: 50px;
  }
</style>
