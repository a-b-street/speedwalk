<script lang="ts">
  import { overpassServer, suggestedServers } from "./overpass";
</script>

<details class="mt-3">
  <summary>Change overpass server…</summary>
  <div class="mt-2">
    {#each suggestedServers as server}
      <div class="form-check">
        <input
          class="form-check-input"
          type="radio"
          name="overpass-server"
          id="server-{server}"
          value={server}
          checked={$overpassServer === server}
          onchange={() => overpassServer.set(server)}
        />
        <label class="form-check-label" for="server-{server}">
          {server}
        </label>
      </div>
    {/each}
  </div>
</details>
