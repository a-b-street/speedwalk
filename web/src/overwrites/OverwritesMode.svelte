<script lang="ts">
  import {
    getOverrides,
    saveOverrides,
    filterSegmentsInRegion,
    type RegionOverrides,
    type AddedCrossingSegment,
  } from "../common/localOverrides";
  import { backend, map, mutationCounter, refreshLoadingScreen } from "../";
  import {
    GeoJSON,
    LineLayer,
    CircleLayer,
    MapEvents,
    Control,
  } from "svelte-maplibre";
  import type { MapMouseEvent } from "maplibre-gl";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import Jumbotron from "../common/Jumbotron.svelte";
  import CollapsibleCard from "../common/CollapsibleCard.svelte";
  import LegendList from "../common/LegendList.svelte";
  import { downloadGeneratedFile } from "svelte-utils";
  import type { FeatureCollection, LineString, Point } from "geojson";
  import { type WayProps, type NodeProps } from "../sidewalks";
  import { roadLineWidth } from "../sidewalks";

  const overwritesLegendItems = [
    { label: "Base data", color: "black", swatchClass: "rectangle" as const },
    {
      label: "Manually added",
      color: "blue",
      swatchClass: "rectangle" as const,
    },
  ];

  /** First click: red dot (start of crossing segment). Second click: blue dot (end). */
  let pointA: { lng: number; lat: number } | null = $state(null);
  let pointB: { lng: number; lat: number } | null = $state(null);
  let overrides: RegionOverrides = $state({ version: 1, addedCrossings: [] });
  let overwritesApplied = $state(true);
  let appliedCount = $state(0);
  let nodes: FeatureCollection<Point, NodeProps> = $state.raw({
    type: "FeatureCollection",
    features: [],
  });
  let ways: FeatureCollection<LineString, WayProps> = $state.raw({
    type: "FeatureCollection",
    features: [],
  });

  $effect(() => {
    $mutationCounter;
    if ($backend) {
      try {
        nodes = JSON.parse($backend.getNodes());
        ways = JSON.parse($backend.getWays());
      } catch (_) {}
    }
  });

  $effect(() => {
    const b = $backend;
    if (!b) {
      appliedCount = 0;
      return;
    }
    getOverrides().then((data) => {
      overrides = data;
      const boundary = JSON.parse(b.getBoundary());
      const list = filterSegmentsInRegion(data.addedCrossings, boundary);
      if (overwritesApplied && list.length > 0 && appliedCount === 0) {
        applyAll(list);
      }
    });
  });

  const inRegionSegments = $derived.by(() => {
    if (!$backend) return [];
    try {
      const boundary = JSON.parse($backend.getBoundary());
      return filterSegmentsInRegion(overrides.addedCrossings, boundary);
    } catch {
      return [];
    }
  });

  const crossingWayTags = {
    highway: "footway",
    footway: "crossing",
    crossing: "manual",
  };

  async function applyAll(segments: AddedCrossingSegment[]) {
    if (!$backend) return;
    await refreshLoadingScreen();
    for (const seg of segments) {
      $backend.editAddCrossingSegment(
        seg.start.lng,
        seg.start.lat,
        seg.end.lng,
        seg.end.lat,
        { ...crossingWayTags, ...seg.tags },
      );
      mutationCounter.update((n) => n + 1);
    }
    appliedCount = segments.length;
  }

  async function unapplyAll() {
    if (!$backend || appliedCount === 0) return;
    await refreshLoadingScreen();
    for (let i = 0; i < appliedCount; i++) {
      $backend.editUndo();
      mutationCounter.update((n) => n + 1);
    }
    appliedCount = 0;
  }

  async function toggleApply() {
    if (!$backend) return;
    if (overwritesApplied) {
      await unapplyAll();
      overwritesApplied = false;
    } else {
      await applyAll(inRegionSegments);
      overwritesApplied = true;
    }
  }

  function onMapClick(e: MapMouseEvent) {
    let lng: number;
    let lat: number;
    if (e.lngLat) {
      lng = e.lngLat.lng;
      lat = e.lngLat.lat;
    } else if ($map && e.point) {
      const ll = $map.unproject(e.point);
      lng = ll.lng;
      lat = ll.lat;
    } else {
      return;
    }
    const pt = { lng, lat };
    if (pointA === null) {
      pointA = pt;
    } else if (pointB === null) {
      // Second click: order so red (A) is left, blue (B) is right (by longitude)
      const a = pointA;
      if (pt.lng < a.lng) {
        pointA = pt;
        pointB = a;
      } else {
        pointB = pt;
      }
    } else {
      // Third+ click: replace the point on the same side (left vs right of segment)
      const midLng = (pointA.lng + pointB.lng) / 2;
      if (pt.lng < midLng) {
        pointA = pt;
      } else {
        pointB = pt;
      }
    }
  }

  async function addCrossingSegmentFromDraft() {
    if (!pointA || !pointB || !$backend) return;
    const tags = { ...crossingWayTags };
    await refreshLoadingScreen();
    $backend.editAddCrossingSegment(
      pointA.lng,
      pointA.lat,
      pointB.lng,
      pointB.lat,
      tags,
    );
    mutationCounter.update((n) => n + 1);
    const newEntry: AddedCrossingSegment = {
      id: crypto.randomUUID(),
      start: { lat: pointA.lat, lng: pointA.lng },
      end: { lat: pointB.lat, lng: pointB.lng },
      tags,
    };
    overrides = {
      ...overrides,
      addedCrossings: [...overrides.addedCrossings, newEntry],
    };
    await saveOverrides(overrides);
    appliedCount++;
    pointA = null;
    pointB = null;
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.key !== "a") return;
    if (
      e.target instanceof HTMLInputElement ||
      e.target instanceof HTMLTextAreaElement
    )
      return;
    addCrossingSegmentFromDraft();
    e.preventDefault();
  }

  async function removeAddedCrossing(segment: AddedCrossingSegment) {
    if (!$backend) return;
    const id = segment.id;
    const list = overrides.addedCrossings.filter((s) => s.id !== id);
    const wasApplied =
      inRegionSegments.findIndex((s) => s.id === id) < appliedCount;
    overrides = { ...overrides, addedCrossings: list };
    await saveOverrides(overrides);
    if (wasApplied && $backend) {
      await refreshLoadingScreen();
      for (let i = 0; i < appliedCount; i++) {
        $backend.editUndo();
        mutationCounter.update((n) => n + 1);
      }
      appliedCount = 0;
      const stillInRegion = filterSegmentsInRegion(
        list,
        JSON.parse($backend.getBoundary()),
      );
      for (const seg of stillInRegion) {
        $backend.editAddCrossingSegment(
          seg.start.lng,
          seg.start.lat,
          seg.end.lng,
          seg.end.lat,
          { ...crossingWayTags, ...seg.tags },
        );
        mutationCounter.update((n) => n + 1);
      }
      appliedCount = stillInRegion.length;
    }
  }

  function zoomToSegment(seg: AddedCrossingSegment) {
    const lngs = [seg.start.lng, seg.end.lng];
    const lats = [seg.start.lat, seg.end.lat];
    const pad = 0.0001;
    $map?.fitBounds(
      [
        [Math.min(...lngs) - pad, Math.min(...lats) - pad],
        [Math.max(...lngs) + pad, Math.max(...lats) + pad],
      ],
      { padding: 50, maxZoom: 18 },
    );
  }

  function exportOverwrites() {
    const blob = JSON.stringify(overrides, null, 2);
    downloadGeneratedFile("speedwalk-overwrites.json", blob);
  }

  let importInput: HTMLInputElement;
  function importOverwrites() {
    importInput?.click();
  }

  async function onImportFile(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    const text = await file.text();
    const data = JSON.parse(text) as RegionOverrides & { regionKey?: string };
    const toMerge = data.addedCrossings ?? [];
    overrides = {
      version: 1,
      addedCrossings: [...overrides.addedCrossings, ...toMerge],
    };
    await saveOverrides(overrides);
    input.value = "";
  }

  const appliedList = $derived(inRegionSegments.slice(0, appliedCount));
  const notAppliedList = $derived(inRegionSegments.slice(appliedCount));

  const draftPointsGeoJSON = $derived.by(() => {
    const features: Array<{
      type: "Feature";
      geometry: { type: "Point"; coordinates: [number, number] };
      properties: { dot: string };
    }> = [];
    if (pointA) {
      features.push({
        type: "Feature",
        geometry: { type: "Point", coordinates: [pointA.lng, pointA.lat] },
        properties: { dot: "red" },
      });
    }
    if (pointB) {
      features.push({
        type: "Feature",
        geometry: { type: "Point", coordinates: [pointB.lng, pointB.lat] },
        properties: { dot: "blue" },
      });
    }
    return { type: "FeatureCollection" as const, features };
  });

  const draftLineGeoJSON = $derived.by(() => {
    const a = pointA;
    const b = pointB;
    if (!a || !b) return { type: "FeatureCollection" as const, features: [] };
    return {
      type: "FeatureCollection" as const,
      features: [
        {
          type: "Feature" as const,
          geometry: {
            type: "LineString" as const,
            coordinates: [
              [a.lng, a.lat],
              [b.lng, b.lat],
            ],
          },
          properties: {},
        },
      ],
    };
  });
</script>

<svelte:window onkeydown={onKeyDown} />

<input
  type="file"
  accept=".json"
  bind:this={importInput}
  onchange={onImportFile}
  style="display: none"
/>

<SplitComponent>
  {#snippet left()}
    <Jumbotron
      title="Manual overwrites"
      lead="Modify the network by manually removing geometries and adding junctions. Changes are stored in your browser."
    >
      <p class="small mb-1">
        <strong>Add crossing:</strong>
        First click = red (left), second = blue (right). Click again to move either
        point. Press
        <kbd>a</kbd>
        to add; both points are snapped to the nearest road or sidewalk.
      </p>
      <p class="small mb-0 text-muted">
        The new crossing is a routable segment between the two snapped points on
        the network.
      </p>
    </Jumbotron>

    {#if !$backend}
      <div class="alert alert-warning py-2 small mb-3" role="alert">
        <strong>Load an area first.</strong>
        Add crossing (two clicks +
        <kbd>a</kbd>
        ), apply, and export/import are available after you load data (relation, polygon,
        or file).
      </div>
    {/if}

    <button
      class="btn mb-3 w-100 {overwritesApplied
        ? 'btn-secondary'
        : 'btn-primary'}"
      onclick={toggleApply}
      disabled={!$backend}
    >
      {overwritesApplied
        ? "Unapply manual overwrites from current data"
        : "Apply manual overwrites to current data"}
    </button>

    <CollapsibleCard open={true}>
      {#snippet header()}
        In your current region: {inRegionSegments.length} in storage, {appliedCount}
        applied
      {/snippet}
      {#snippet body()}
        {#if notAppliedList.length > 0}
          <h6 class="mt-2">Could not apply</h6>
          <ul class="list-unstyled small">
            {#each notAppliedList as seg}
              <li class="d-flex align-items-center gap-2 mb-1">
                <span class="text-break small">
                  {seg.start.lat.toFixed(4)},{seg.start.lng.toFixed(4)} → {seg.end.lat.toFixed(
                    4,
                  )},{seg.end.lng.toFixed(4)}
                </span>
                <button
                  type="button"
                  class="btn btn-link p-0 small text-primary"
                  onclick={() => zoomToSegment(seg)}
                >
                  Zoom
                </button>
                <button
                  class="btn btn-sm btn-outline-danger"
                  onclick={() => removeAddedCrossing(seg)}
                >
                  Remove
                </button>
              </li>
            {/each}
          </ul>
        {/if}
        {#if appliedList.length > 0}
          <h6 class="mt-2">Applied</h6>
          <ul class="list-unstyled small">
            {#each appliedList as seg}
              <li class="d-flex align-items-center gap-2 mb-1">
                <span class="text-break small">
                  {seg.start.lat.toFixed(4)},{seg.start.lng.toFixed(4)} → {seg.end.lat.toFixed(
                    4,
                  )},{seg.end.lng.toFixed(4)}
                </span>
                <button
                  type="button"
                  class="btn btn-link p-0 small text-primary"
                  onclick={() => zoomToSegment(seg)}
                >
                  Zoom
                </button>
                <button
                  class="btn btn-sm btn-outline-danger"
                  onclick={() => removeAddedCrossing(seg)}
                >
                  Remove
                </button>
              </li>
            {/each}
          </ul>
        {/if}
        {#if inRegionSegments.length === 0}
          <p class="text-muted small">
            {#if overrides.addedCrossings.length === 0}
              No manual crossings yet.
            {:else}
              No overwrites in this region ({overrides.addedCrossings.length} total
              in storage).
            {/if}
          </p>
        {/if}
      {/snippet}
    </CollapsibleCard>

    <div class="mt-3">
      <button
        class="btn btn-secondary btn-sm me-1"
        onclick={exportOverwrites}
        disabled={!$backend}
      >
        Export overwrites
      </button>
      <button
        class="btn btn-secondary btn-sm"
        onclick={importOverwrites}
        disabled={!$backend}
      >
        Import overwrites
      </button>
    </div>
  {/snippet}

  {#snippet main()}
    <MapEvents onclick={onMapClick} />

    {#if pointA || pointB}
      <GeoJSON data={draftPointsGeoJSON} generateId>
        <CircleLayer
          id="overwrites-draft-dots"
          paint={{
            "circle-radius": 12,
            "circle-color": [
              "match",
              ["get", "dot"],
              "red",
              "#e74c3c",
              "blue",
              "#3498db",
              "#999",
            ],
            "circle-stroke-width": 2,
            "circle-stroke-color": "#fff",
          }}
        />
      </GeoJSON>
    {/if}
    {#if pointA && pointB}
      <GeoJSON data={draftLineGeoJSON} generateId>
        <LineLayer
          id="overwrites-draft-line"
          paint={{
            "line-width": 3,
            "line-color": "#9b59b6",
            "line-dasharray": [2, 2],
          }}
        />
      </GeoJSON>
    {/if}

    {#if $backend}
      <GeoJSON data={ways} generateId>
        <LineLayer
          id="overwrites-ways"
          paint={{
            "line-width": roadLineWidth(0),
            "line-color": [
              "case",
              ["get", "is_manual_crossing"],
              "blue",
              "black",
            ],
          }}
        />
      </GeoJSON>
      <GeoJSON data={nodes} generateId>
        <CircleLayer
          id="overwrites-nodes"
          paint={{
            "circle-radius": 6,
            "circle-color": [
              "case",
              ["get", "is_manual_crossing"],
              "blue",
              "black",
            ],
          }}
        />
      </GeoJSON>
    {/if}

    <Control position="top-right">
      <CollapsibleCard>
        {#snippet header()}Legend{/snippet}
        {#snippet body()}
          <LegendList items={overwritesLegendItems} />
        {/snippet}
      </CollapsibleCard>
    </Control>

    {#if pointA && pointB}
      <div
        class="position-absolute top-0 start-50 translate-middle-x mt-2 px-2 py-1 bg-light border rounded small"
      >
        Red = left, blue = right. Click again to move left or right point. Press <kbd
        >
          a
        </kbd>
        to add crossing.
      </div>
    {:else if pointA}
      <div
        class="position-absolute top-0 start-50 translate-middle-x mt-2 px-2 py-1 bg-light border rounded small"
      >
        First point set (red, left). Click for second point (blue, right).
      </div>
    {/if}
  {/snippet}
</SplitComponent>
