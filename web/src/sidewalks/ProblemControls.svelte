<script lang="ts">
  import CollapsibleCard from "../common/CollapsibleCard.svelte";
  import type { FeatureCollection, LineString, Point } from "geojson";
  import type { NodeProps, WayProps } from "./";
  import { map } from "../";
  import { bbox } from "svelte-utils/map";

  export let nodes: FeatureCollection<Point, NodeProps>;
  export let ways: FeatureCollection<LineString, WayProps>;
  export let drawProblems: FeatureCollection;

  let nodeProblems = [
    "missing crossing node",
    "separate sidewalks should be continued here",
  ];
  let wayProblems = [
    "missing footway=crossing",
    "possible separate sidewalk near way without it tagged",
    "sidewalk=separate is ambiguous about the side",
    "sidewalk:left and sidewalk:right should each be tagged as separate or no",
  ];

  $: problemCounts = countProblems(nodes, ways);

  let show = "";
  let currentProblemIndex = 0;

  $: drawProblems = filterProblems(nodes, ways, show);
  $: if (show) {
    // Reset index when problem type changes
    currentProblemIndex = 0;
  }
  $: if (
    currentProblemIndex >= drawProblems.features.length &&
    drawProblems.features.length > 0
  ) {
    // Reset index if it's out of bounds
    currentProblemIndex = 0;
  }

  function countProblems(_a: any, _b: any): Record<string, number> {
    let counts = {} as Record<string, number>;
    for (let x of [...nodeProblems, ...wayProblems]) {
      counts[x] = 0;
    }

    for (let f of [...nodes.features, ...ways.features]) {
      for (let problem of f.properties.problems) {
        counts[problem.note] += 1;
      }
    }

    return counts;
  }

  function filterProblems(_a: any, _b: any, _c: any): FeatureCollection {
    let gj = {
      type: "FeatureCollection" as const,
      features: [],
    } as FeatureCollection;
    for (let f of [...nodes.features, ...ways.features]) {
      if (f.properties.problems.some((p: any) => p.note == show)) {
        gj.features.push(f);
      }
    }
    return gj;
  }

  function pickNextProblem() {
    if (!$map || !show || drawProblems.features.length === 0) {
      return;
    }

    // Cycle through problems
    const feature = drawProblems.features[currentProblemIndex];
    currentProblemIndex =
      (currentProblemIndex + 1) % drawProblems.features.length;

    // Focus map on the feature
    try {
      if (feature.geometry.type === "Point") {
        // For points, use flyTo with a reasonable zoom level
        const coords = feature.geometry.coordinates;
        $map.flyTo({
          center: [coords[0], coords[1]],
          zoom: 19,
          animate: true,
        });
      } else {
        // For lines, use fitBounds with padding
        const bounds = bbox(feature);
        $map.fitBounds(bounds, { animate: true, padding: 50 });
      }
    } catch (err) {
      console.error("Error focusing on problem:", err);
    }
  }
</script>

<CollapsibleCard>
  {#snippet header()}Problems{/snippet}
  {#snippet body()}
    <select class="form-select" bind:value={show}>
      <option value="">Show a type of problem</option>
      {#each Object.entries(problemCounts) as [problem, count]}
        <option value={problem} disabled={count == 0}>
          {problem} ({count})
        </option>
      {/each}
    </select>

    {#if show}
      <div class="problem-header">
        <p class="problem-count">{problemCounts[show]} problems</p>
        {#if drawProblems.features.length > 0}
          <button class="btn btn-sm btn-primary" on:click={pickNextProblem}>
            Pick next
          </button>
        {/if}
      </div>
    {/if}

    {#if show == "missing crossing node"}
      <p>
        When a crossing way hits a road, the node should be tagged as a
        crossing. Instructions to fix are TODO.
      </p>
    {:else if show == "separate sidewalks should be continued here"}
      <p>
        Here are places where part of a road has separate sidewalks tagged, but
        then the next part of the road doesn't. It's best to be consistent about
        separate sidewalks along the entire length of a road. To fix:
      </p>
      <ol>
        <li>
          Go map separate sidewalks along the entire road (in another editor)
        </li>
        <li>
          Update the road tagging to indicate those separate sidewalks (here or
          in another editor)
        </li>
        <li>Refresh the data here to verify</li>
      </ol>
      <p>
        (This problem is detected when the road names are the same, so there are
        some false positives you can ignore.)
      </p>
    {:else if show == "missing footway=crossing"}
      <p>
        Here are footways with a crossing node, but the way needs to be split
        and tagged as <i>footway=crossing</i>
        .
      </p>
      <ol>
        <li>Open in another editor</li>
        <li>Split the way around the crossing</li>
        <li>
          Tag <i>footway=crossing</i>
          on the smaller segment
        </li>
        <li>Refresh the data here to verify</li>
      </ol>
    {:else if show == "possible separate sidewalk near way without it tagged"}
      <p>
        These roads aren't tagged as having separate sidewalks, but it looks
        like there's a parallel separate sidewalk already mapped.
      </p>
      <ol>
        <li>
          Check each segment. If separate sidewalks are already there, update
          the tagging here.
        </li>
        <li>
          If the separate sidewalks don't cover the entire length of the road,
          ideally go finish drawing the separate sidewalks in another tool, then
          update the tagging on the road.
        </li>
      </ol>
      <p>There are false positives. Check the full length of the road.</p>
    {:else if show == "sidewalk=separate is ambiguous about the side"}
      <p>
        <i>sidewalk=separate</i>
        is ambiguous. Update the tagging to specify if there are separate sidewalks
        on both sides or just one.
      </p>
    {:else if show == "sidewalk:left and sidewalk:right should each be tagged as separate or no"}
      <p>
        These roads have separate sidewalks tagged on one side, but the other
        side is unspecified or not drawn separately. Be consistent on each road
        and use another editor to draw separate sidewalks on both sides.
      </p>{/if}
  {/snippet}
</CollapsibleCard>

<style>
  .problem-header {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 10px;
    margin-bottom: 10px;
  }

  .problem-count {
    margin: 0;
  }
</style>
