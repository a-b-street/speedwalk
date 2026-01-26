<script lang="ts">
  import { networkFilter } from "../";
  import { Checkbox } from "svelte-utils";
</script>

<div class="card">
  <div class="card-body">
    <div class="form-check">
      <label class="form-check-label">
        <input
          class="form-check-input"
          type="radio"
          bind:group={$networkFilter.include}
          value="Everything"
        />
        Everything
      </label>
    </div>

    <div class="form-check">
      <label class="form-check-label">
        <input
          class="form-check-input"
          type="radio"
          bind:group={$networkFilter.include}
          value="OnlyExplicitFootways"
        />
        Only footways
      </label>
    </div>

    <div class="form-check mb-3">
      <label class="form-check-label">
        <input
          class="form-check-input"
          type="radio"
          bind:group={$networkFilter.include}
          value="RouteableNetwork"
        />
        Anything routeable for walking
      </label>
    </div>

    <Checkbox bind:checked={$networkFilter.ignore_deadends}>
      Ignore dead ends under 10m and disconnected segments under 100m
    </Checkbox>
  </div>
</div>
