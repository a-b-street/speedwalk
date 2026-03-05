<script lang="ts">
  import { networkFilter } from "../";
  import { Checkbox } from "svelte-utils";
</script>

<div class="pt-2">
  <div class="form-check mb-1">
    <label class="form-check-label">
      <input
        class="form-check-input"
        type="radio"
        bind:group={$networkFilter.include}
        value="Everything"
      />
      Full network
    </label>
  </div>

  <div class="form-check mb-1">
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
      Anything routeable for walking (Default)
    </label>
  </div>

  <div class="border-top pt-2 mt-2">
    <Checkbox bind:checked={$networkFilter.ignore_deadends}>
      Ignore dead ends under 10m and disconnected segments under 100m
    </Checkbox>
  </div>
</div>
