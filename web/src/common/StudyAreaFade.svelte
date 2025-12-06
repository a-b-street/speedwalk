<script lang="ts">
  import { backend } from "../";
  import { GeoJSON, FillLayer } from "svelte-maplibre";

  let boundary = JSON.parse($backend!.getBoundary());
  let data = {
    type: "Feature" as const,
    properties: {},
    geometry: {
      type: "Polygon" as const,
      coordinates: [
        [
          [180.0, 90.0],
          [-180.0, 90.0],
          [-180.0, -90.0],
          [180.0, -90.0],
          [180.0, 90.0],
        ],
        boundary.geometry.coordinates[0],
      ],
    },
  };
</script>

<GeoJSON {data}>
  <FillLayer paint={{ "fill-color": "black", "fill-opacity": 0.3 }} />
</GeoJSON>
