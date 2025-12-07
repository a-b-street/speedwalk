<script lang="ts">
  import CollapsibleCard from "../common/CollapsibleCard.svelte";
  import type { FeatureCollection, LineString, Point } from "geojson";
  import type { NodeProps, WayProps } from "./";

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

  $: drawProblems = filterProblems(nodes, ways, show);

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
</script>

<CollapsibleCard>
  <div slot="header">Problems</div>
  <div slot="body">
    <select class="form-select" bind:value={show}>
      <option value="">Show a type of problem</option>
      {#each Object.entries(problemCounts) as [problem, count]}
        <option value={problem} disabled={count == 0}>
          {problem} ({count})
        </option>
      {/each}
    </select>

    {#if show == "missing crossing node"}
      <p>
        Here are nodes on crossing ways that aren't tagged as a crossing. You
        should fix by...
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
    {:else if show == "missing footway=crossing"}
      <p>TODO</p>
    {:else if show == "possible separate sidewalk near way without it tagged"}
      <p>TODO</p>
    {:else if show == "sidewalk=separate is ambiguous about the side"}
      <p>TODO</p>
    {:else if show == "sidewalk:left and sidewalk:right should each be tagged as separate or no"}
      <p>TODO</p>
    {/if}
  </div>
</CollapsibleCard>
