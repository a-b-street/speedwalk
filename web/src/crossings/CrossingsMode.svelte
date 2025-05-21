<script lang="ts">
  import { backend, mutationCounter } from "../";
  import { SplitComponent } from "svelte-utils/two_column_layout";
  import { Control } from "svelte-maplibre";
  import type { Map } from "maplibre-gl";

  export let map: Map;
</script>

<SplitComponent>
  <div slot="sidebar"></div>

  <div slot="map">
    <Control position="top-right">
      <div style:background="white" style:width="200px" style:padding="8px">
        Stuff
      </div>
    </Control>
  </div>
</SplitComponent>
