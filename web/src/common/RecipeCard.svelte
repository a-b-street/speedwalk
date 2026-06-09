<script lang="ts">
  import {
    backend,
    mutationCounter,
    networkFilter,
    pendingRecipe,
    recipeSteps,
    crossingScopeBulk,
    onlyMajorRoadsBulk,
    includeCrossingNoBulk,
    refreshLoadingScreen,
  } from "../";
  import { stepLabel } from "../recipe";
  import type { RecipeStep } from "../recipe";
  import { applyStoredOverrides } from "./applyOverridesRecipe";

  const defaultCrossingOptions = {
    only_major_roads: true,
    ignore_utility_roads: true,
    ignore_cycleways: true,
    ignore_footways: true,
    ignore_roundabouts: true,
    ignore_motorways: true,
    max_distance: 40,
  };

  let applyLoading = $state("");

  async function applyRecipe() {
    const recipe = $pendingRecipe;
    if (!recipe) return;
    pendingRecipe.set(null);

    for (const step of recipe.steps) {
      applyLoading = stepLabel(step);
      await refreshLoadingScreen();
      try {
        await executeStep(step);
        $mutationCounter++;
      } catch (err) {
        window.alert(`Error in step "${stepLabel(step)}": ${err}`);
        break;
      }
    }
    applyLoading = "";
  }

  async function executeStep(step: RecipeStep) {
    switch (step.op) {
      case "generateMissingCrossings": {
        crossingScopeBulk.set(step.scope);
        const options = {
          ...defaultCrossingOptions,
          only_major_roads: step.scope === "major",
          ignore_utility_roads: step.scope !== "all",
        };
        $backend!.editGenerateMissingCrossings(options);
        recipeSteps.update((s) => [...s, step]);
        break;
      }
      case "makeAllSidewalks":
        onlyMajorRoadsBulk.set(step.onlyMajor);
        $backend!.editMakeAllSidewalks(step.onlyMajor);
        recipeSteps.update((s) => [...s, step]);
        break;
      case "connectAllCrossings":
        includeCrossingNoBulk.set(step.includeCrossingNo);
        $backend!.editConnectAllCrossings(step.includeCrossingNo);
        recipeSteps.update((s) => [...s, step]);
        break;
      case "assumeTags":
        $backend!.editAssumeTags(step.driveOnLeft);
        recipeSteps.update((s) => [...s, step]);
        break;
      case "applyOverrides":
        await applyStoredOverrides($backend!);
        recipeSteps.update((s) => [...s, step]);
        break;
      case "applyMaxspeed":
        $backend!.editApplyMaxspeed();
        recipeSteps.update((s) => [...s, step]);
        break;
      case "applyCrossingNodeTags":
        $backend!.editApplyCrossingNodeTags();
        recipeSteps.update((s) => [...s, step]);
        break;
      case "setNetworkFilter":
        networkFilter.set({
          include: step.include,
          ignore_deadends: step.ignore_deadends,
        });
        recipeSteps.update((s) => [...s, step]);
        break;
    }
  }
</script>

{#if $pendingRecipe}
  <div class="card mb-3 border-info">
    <div class="card-header bg-info-subtle text-info-emphasis">
      <strong>Recipe</strong> — {$pendingRecipe.steps.length}
      {$pendingRecipe.steps.length === 1 ? "step" : "steps"}
    </div>
    <div class="card-body py-2">
      <ol class="mb-2 ps-3">
        {#each $pendingRecipe.steps as step}
          <li class="small">{stepLabel(step)}</li>
        {/each}
      </ol>
      {#if applyLoading}
        <p class="small text-muted mb-2">Applying: {applyLoading}…</p>
      {/if}
      <button
        class="btn btn-sm btn-primary me-2"
        onclick={applyRecipe}
        disabled={!!applyLoading}
      >
        Apply all steps
      </button>
      <button
        class="btn btn-sm btn-outline-secondary"
        onclick={() => pendingRecipe.set(null)}
        disabled={!!applyLoading}
      >
        Dismiss
      </button>
    </div>
  </div>
{/if}
