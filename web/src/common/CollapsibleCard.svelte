<script lang="ts">
  import { v4 as uuidv4 } from "uuid";

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
        <slot name="header"></slot>
      </button>
    </div>
    <div {id} class="accordion-collapse collapse show">
      <div class="accordion-body">
        <slot name="body"></slot>
      </div>
    </div>
  </div>
</div>
