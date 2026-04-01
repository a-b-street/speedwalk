<script lang="ts">
  import {
    LineLayer,
    GeoJSON,
    hoverStateFilter,
    Popup,
    Control,
  } from "svelte-maplibre";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { backend, mutationCounter } from "../";
  import type { FeatureCollection } from "geojson";
  import { emptyGeojson } from "svelte-utils/map";
  import CollapsibleCard from "../common/CollapsibleCard.svelte";
  import Jumbotron from "../common/Jumbotron.svelte";
  import LegendList from "../common/LegendList.svelte";

  const legendItems: { color: string; label: string; swatchClass: "rectangle" }[] = [
    { color: "#2ecc71", label: "≤ 30 km/h", swatchClass: "rectangle" },
    { color: "#f1c40f", label: "≤ 50 km/h", swatchClass: "rectangle" },
    { color: "#e74c3c", label: "> 50 km/h", swatchClass: "rectangle" },
    { color: "#aaaaaa", label: "No maxspeed", swatchClass: "rectangle" },
  ];

  let crossings = $derived.by(() => {
    $mutationCounter;
    if (!$backend) {
      return emptyGeojson();
    }
    const gj = JSON.parse($backend.getWays()) as FeatureCollection;
    gj.features = gj.features.filter(
      (f) => f.properties!.kind === "Crossing",
    );
    for (const f of gj.features) {
      const tags = f.properties!.tags ?? {};
      const ms: string | undefined = tags.maxspeed;
      f.properties!.maxspeed = ms ?? null;
      f.properties!.maxspeed_num = ms ? parseFloat(ms) : null;
    }
    return gj;
  });

  let crossingsWithMaxspeed = $derived(
    crossings.features.filter((f) => f.properties!.maxspeed !== null).length,
  );

  let crossingsWithoutMaxspeed = $derived(
    crossings.features.length - crossingsWithMaxspeed,
  );

  let working = $state(false);

  function applyMaxspeed() {
    working = true;
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        $backend!.editApplyMaxspeed();
        $mutationCounter++;
        working = false;
      });
    });
  }
</script>

<SplitComponent>
  {#snippet left()}
    <Jumbotron
      title="Maxspeed"
      lead="Enriches crossing ways with the maxspeed attribute of the crossed road."
    >
      <p class="mb-0 small text-muted">
        {crossingsWithMaxspeed.toLocaleString()} of {crossings.features.length.toLocaleString()}
        crossings already have a maxspeed attribute.
      </p>
      {#if crossingsWithoutMaxspeed > 0}
        <p class="mb-0 small text-muted">
          {crossingsWithoutMaxspeed.toLocaleString()} crossings have no maxspeed — road has no maxspeed data.
        </p>
      {/if}
    </Jumbotron>

    <button class="btn btn-primary mb-3" onclick={applyMaxspeed} disabled={working}>
      {#if working}
        <span class="spinner-border spinner-border-sm me-2" role="status" aria-hidden="true"></span>
        Applying…
      {:else}
        Apply maxspeed to all crossings
      {/if}
    </button>
  {/snippet}

  {#snippet main()}
    <GeoJSON data={crossings} generateId>
      <LineLayer
        manageHoverState
        paint={{
          "line-width": 5,
          "line-opacity": hoverStateFilter(0.6, 1.0),
          "line-color": [
            "case",
            ["==", ["get", "maxspeed_num"], null],
            "#aaaaaa",
            ["<=", ["get", "maxspeed_num"], 30],
            "#2ecc71",
            ["<=", ["get", "maxspeed_num"], 50],
            "#f1c40f",
            "#e74c3c",
          ],
        }}
        hoverCursor="pointer"
      >
        <Popup openOn="hover">
          {#snippet children({ data })}
            {@const props = data?.properties ?? {}}
            <table class="table table-bordered table-sm mb-0">
              <tbody>
                <tr>
                  <td>maxspeed</td>
                  <td>{props.maxspeed ?? "–"}</td>
                </tr>
                {#if props.tags}
                  {@const tags = props.tags}
                  {#if tags.crossing}
                    <tr>
                      <td>crossing</td>
                      <td>{tags.crossing}</td>
                    </tr>
                  {/if}
                {/if}
              </tbody>
            </table>
          {/snippet}
        </Popup>
      </LineLayer>
    </GeoJSON>

    <Control position="top-right">
      <CollapsibleCard>
        {#snippet header()}Legend{/snippet}
        {#snippet body()}
          <LegendList items={legendItems} />
        {/snippet}
      </CollapsibleCard>
    </Control>
  {/snippet}
</SplitComponent>
