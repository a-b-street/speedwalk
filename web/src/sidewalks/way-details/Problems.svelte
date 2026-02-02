<script lang="ts">
  import type { FeatureCollection, Geometry } from "geojson";
  import { Checkbox, QualitativeLegend } from "svelte-utils";

  let {
    problems,
    drawProblemDetails,
    showProblemDetails = $bindable(),
  }: {
    problems: Array<{ note: string }>;
    drawProblemDetails: FeatureCollection<
      Geometry,
      { label: string; color: string }
    >;
    showProblemDetails: boolean;
  } = $props();
</script>

{#if problems.length}
  {@const headerProblem =
    problems.find(
      (p) => p.note === "possible separate sidewalk near way without it tagged",
    ) || problems[0]}
  {@const remainingProblems = problems.filter(
    (p) => p.note !== headerProblem.note,
  )}
  <div class="alert alert-warning">
    <h5 class="alert-heading d-flex align-items-center">
      <div class="flex-shrink-0 me-2">
        <i class="fa-solid fa-triangle-exclamation"></i>
      </div>
      <div class="flex-grow-1">
        {headerProblem.note}
      </div>
    </h5>
    {#if remainingProblems.length}
      {#each remainingProblems as problem}
        <p class="mb-0">{problem.note}</p>
      {/each}
    {/if}

    {#if drawProblemDetails.features.length}
      <Checkbox bind:checked={showProblemDetails}>
        Highlight problem on map
      </Checkbox>

      {#if showProblemDetails}
        <QualitativeLegend
          labelColors={Object.fromEntries(
            drawProblemDetails.features.map((f) => [
              f.properties.label,
              f.properties.color,
            ]),
          )}
          itemsPerRow={1}
        />
      {/if}
    {/if}
  </div>
{/if}

<style>
  :global(.alert .color-swatch) {
    flex-shrink: 0;
  }
</style>
