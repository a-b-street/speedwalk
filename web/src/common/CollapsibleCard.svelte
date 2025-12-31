<script lang="ts">
  import type { Snippet } from "svelte";
  import { v4 as uuidv4 } from "uuid";

  interface Props {
    header: Snippet;
    body: Snippet;
    open?: boolean;
  }
  let { header, body, open = true }: Props = $props();

  let id = uuidv4();
</script>

<div class="accordion mb-3">
  <div class="accordion-item">
    <div class="accordion-header">
      <button
        class="accordion-button"
        type="button"
        data-bs-toggle="collapse"
        data-bs-target={`#${id}`}
        aria-expanded="true"
        aria-controls={id}
      >
        {@render header()}
      </button>
    </div>
    <div {id} class="accordion-collapse collapse" class:show={open}>
      <div class="accordion-body">
        {@render body()}
      </div>
    </div>
  </div>
</div>
