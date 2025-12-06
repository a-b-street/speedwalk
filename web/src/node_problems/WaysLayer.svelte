<script lang="ts">
  // TODO Figure out what things are common
  import {
    roadLineWidth,
    colors,
  } from "../sidewalks";
  import { backend } from "../";
  import { constructMatchExpression } from "svelte-utils/map";
  import { GeoJSON, LineLayer } from 'svelte-maplibre';

  let ways = JSON.parse($backend!.getWays());
</script>

    <GeoJSON data={ways}>
      <LineLayer
        id="ways"
        beforeId="Road labels"
        manageHoverState
        paint={{
          "line-width": roadLineWidth(0),
          "line-color": constructMatchExpression(
            ["get", "kind"],
            colors,
            "cyan",
          ),
          "line-opacity": 0.5,
        }}
      />
    </GeoJSON>
