<script lang="ts">
  import { map } from "../";
  import ReportProblemModal from "./ReportProblemModal.svelte";

  let reportProblemRef: ReportProblemModal | null = null;
</script>

{#if $map}
  <div class="report-problem-wrapper">
    <button
      class="btn btn-secondary report-problem-btn"
      on:click={() => {
        reportProblemRef?.triggerStart();
      }}
    >
      <i class="fa-solid fa-triangle-exclamation"></i>
      <span class="report-problem-text">Report problem</span>
    </button>
  </div>
{/if}

<ReportProblemModal bind:this={reportProblemRef} />

<style>
  .report-problem-wrapper {
    position: relative;
  }

  .report-problem-btn {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .report-problem-text {
    width: 0;
    opacity: 0;
    overflow: hidden;
    transition: opacity 0.2s, width 0.2s;
    white-space: nowrap;
  }

  .report-problem-wrapper:hover .report-problem-text {
    width: auto;
    opacity: 1;
  }
</style>
