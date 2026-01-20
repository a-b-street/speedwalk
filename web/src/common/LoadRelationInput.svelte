<script lang="ts">
  import { Loading } from "svelte-utils";
  import { loadRelationAndCreateSpeedwalk } from "./relationLoader";

  let {
    onSuccess,
  }: {
    onSuccess?: () => void;
  } = $props();

  let relationId = $state("");
  let loading = $state("");

  let isValidRelationId = $derived.by(() => {
    const trimmed = relationId.trim();
    const parsed = parseInt(trimmed);
    return trimmed !== "" && !isNaN(parsed) && parsed > 0;
  });

  async function handleLoad() {
    const id = parseInt(relationId.trim());
    if (!id || id <= 0) {
      window.alert("Please enter a valid relation ID");
      return;
    }

    try {
      loading = "Loading relation...";
      await loadRelationAndCreateSpeedwalk(id);
      // Success - backend is set by loadRelationAndCreateSpeedwalk
      onSuccess?.();
    } catch (err) {
      window.alert(`Failed to load relation: ${err}`);
    } finally {
      loading = "";
    }
  }
</script>

<Loading {loading} />

<div class="mt-3">
  <p class="fst-italic my-3">or…</p>
  <div class="mb-2">
    <label for="relation-id" class="form-label">Load from relation ID</label>
    <div class="input-group">
      <input
        id="relation-id"
        type="text"
        inputmode="numeric"
        pattern="[0-9]*"
        class="form-control"
        placeholder="e.g., 1310102"
        bind:value={relationId}
        onkeydown={(e) => {
          if (e.key === "Enter" && isValidRelationId) {
            handleLoad();
          }
        }}
        onpaste={(e) => {
          // Handle paste to ensure value updates
          setTimeout(() => {
            relationId = relationId.trim();
          }, 0);
        }}
      />
      <button
        class="btn btn-primary"
        type="button"
        onclick={handleLoad}
        disabled={!isValidRelationId}
      >
        Load
      </button>
    </div>
  </div>
</div>
