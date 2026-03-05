<script lang="ts">
  import { QualitativeLegend } from "svelte-utils";
  import { crossingLegendColors } from "./legend";
</script>

<!-- Crossing legend: uses shared QualitativeLegend (label + color swatch list) -->
<QualitativeLegend
  labelColors={crossingLegendColors}
  itemsPerRow={1}
  swatchClass="circle"
/>
