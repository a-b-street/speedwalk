<script lang="ts">
  import {
    getOverrides,
    saveOverrides,
    filterSegmentsInBoundary,
  } from "../common/localOverrides";
  import {
    type AddedCrossingSegment,
    type ManualOverrides,
    isValidSegment,
    manualOverridesSchema,
    snappedSegmentSchema,
  } from "../common/overridesSchema";
  import {
    backend,
    map,
    mutationCounter,
    networkFilter,
    refreshLoadingScreen,
  } from "../";
  import { emptyGeojson } from "svelte-utils/map";
  import {
    GeoJSON,
    LineLayer,
    CircleLayer,
    MapEvents,
    Control,
    Marker,
  } from "svelte-maplibre";
  import type { MapMouseEvent } from "maplibre-gl";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import Jumbotron from "../common/Jumbotron.svelte";
  import CollapsibleCard from "../common/CollapsibleCard.svelte";
  import FilterNetworkCard from "../common/FilterNetworkCard.svelte";
  import LegendList from "../common/LegendList.svelte";
  import { downloadGeneratedFile, Loading } from "svelte-utils";
  import type { FeatureCollection, Point } from "geojson";
  import { type NodeProps } from "../sidewalks";
  import { roadLineWidth } from "../sidewalks";
  import { MAPILLARY_PIN_LAYER_IDS_LIST } from "../common/mapillaryLayers";

  const overridesLegendItems = [
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
  let loading = $state("");
  let applyError = $state("");
  let overrides: ManualOverrides = $state({ version: 1, addedCrossings: [] });
  let overridesApplied = $state(true);
  let appliedCount = $state(0);
  let nodes: FeatureCollection<Point, NodeProps> = $state.raw({
    type: "FeatureCollection",
    features: [],
  });

  $effect(() => {
    $mutationCounter;
    if ($backend) {
      try {
        nodes = JSON.parse($backend.getNodes());
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
      const list = filterSegmentsInBoundary(data.addedCrossings, boundary);
      if (overridesApplied && list.length > 0 && appliedCount === 0) {
        applyAll(list);
      }
    });
  });

  const segmentsInLoadedArea = $derived.by(() => {
    if (!$backend) return [];
    try {
      const boundary = JSON.parse($backend.getBoundary());
      return filterSegmentsInBoundary(overrides.addedCrossings, boundary);
    } catch {
      return [];
    }
  });

  /** Filtered network (respects Filter network options); used for the map line layer. Re-runs when edits change so map stays in sync after apply/unapply. */
  const filteredNetworkGeoJSON = $derived.by(() => {
    $mutationCounter; // depend on edits so map updates after apply/unapply
    if (!$backend) return emptyGeojson();
    try {
      const f = $networkFilter;
      return JSON.parse($backend.exportNetwork(f));
    } catch {
      return emptyGeojson();
    }
  });

  /** Nodes that appear as endpoints of edges in the filtered network, or are manual crossings (always show those). */
  const filteredNodesGeoJSON = $derived.by(() => {
    const net = filteredNetworkGeoJSON;
    const fc = nodes;
    const ids = new Set<number>();
    if (net.features?.length) {
      for (const edge of net.features) {
        const p = edge.properties as
          | { node1?: number; node2?: number }
          | undefined;
        if (p?.node1 != null) ids.add(p.node1);
        if (p?.node2 != null) ids.add(p.node2);
      }
    }
    const features = fc.features.filter((f) => {
      const p = f.properties as
        | { id?: number; is_manual_crossing?: boolean }
        | undefined;
      return ids.has(p?.id as number) || p?.is_manual_crossing === true;
    });
    return { type: "FeatureCollection" as const, features };
  });

  const crossingWayTags = {
    highway: "footway",
    footway: "crossing",
    crossing: "manual",
  };

  async function applyAll(segments: AddedCrossingSegment[]) {
    if (!$backend) return;
    applyError = "";
    loading = "Applying overrides";
    await refreshLoadingScreen();
    try {
      for (const seg of segments) {
        if (!isValidSegment(seg)) continue;
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
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      applyError =
        msg ||
        "Failed to apply override (e.g. could not snap to road or sidewalk)";
    } finally {
      loading = "";
    }
  }

  async function unapplyAll() {
    if (!$backend || appliedCount === 0) return;
    loading = "Unapplying overrides";
    await refreshLoadingScreen();
    try {
      for (let i = 0; i < appliedCount; i++) {
        $backend.editUndo();
        mutationCounter.update((n) => n + 1);
      }
      appliedCount = 0;
    } finally {
      loading = "";
    }
  }

  async function toggleApply() {
    if (!$backend) return;
    if (overridesApplied) {
      await unapplyAll();
      overridesApplied = false;
    } else {
      await applyAll(segmentsInLoadedArea);
      overridesApplied = true;
    }
  }

  /** True when any current draft point is outside the viewport (next click acts as first click). */
  function anyDraftPointOutsideViewport(): boolean {
    if (!$map || (!pointA && !pointB)) return false;
    const b = $map.getBounds();
    if (pointA && !b.contains([pointA.lng, pointA.lat])) return true;
    if (pointB && !b.contains([pointB.lng, pointB.lat])) return true;
    return false;
  }

  function onMapClick(e: MapMouseEvent) {
    // Ignore clicks on draft markers (they are draggable; map click would otherwise move points).
    if (
      (e.originalEvent?.target as Element)?.closest?.(".overrides-draft-marker")
    ) {
      return;
    }
    // Do not set override marker when clicking a Mapillary pin (only when clicking the map).
    // Mapillary layers are conditional; only query layers that exist in the current style.
    if ($map && e.point) {
      const style = $map.getStyle();
      const existingIds = style?.layers
        ? new Set(style.layers.map((l) => l.id))
        : new Set<string>();
      const layersToQuery = MAPILLARY_PIN_LAYER_IDS_LIST.filter((id) =>
        existingIds.has(id),
      );
      const mapillaryFeatures =
        layersToQuery.length > 0
          ? $map.queryRenderedFeatures(e.point, { layers: layersToQuery })
          : [];
      if (mapillaryFeatures.length > 0) {
        return;
      }
    }
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
    // If any draft point is outside the viewport, start fresh (e.g. one accidental click then zoom away).
    if ((pointA || pointB) && anyDraftPointOutsideViewport()) {
      pointA = pt;
      pointB = null;
      return;
    }
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
    loading = "Adding manual crossing";
    await refreshLoadingScreen();
    try {
      const a = { lng: pointA.lng, lat: pointA.lat };
      const b = { lng: pointB.lng, lat: pointB.lat };
      console.debug(
        "[Overrides] Adding manual crossing: pointA =",
        a,
        "pointB =",
        b,
      );
      const rawSnapped = JSON.parse(
        $backend.snapCrossingSegment(
          pointA.lng,
          pointA.lat,
          pointB.lng,
          pointB.lat,
        ),
      );
      const parsed = snappedSegmentSchema.safeParse(rawSnapped);
      if (!parsed.success) {
        console.warn(
          "[Overrides] snapCrossingSegment returned invalid shape:",
          rawSnapped,
          parsed.error,
        );
        applyError =
          "Could not snap to road or sidewalk. Ensure both points are near a road or footway.";
        return;
      }
      const { start, end } = parsed.data;
      $backend.editAddCrossingSegment(
        start.lng,
        start.lat,
        end.lng,
        end.lat,
        tags,
      );
      mutationCounter.update((n) => n + 1);
      const newEntry: AddedCrossingSegment = {
        id: crypto.randomUUID(),
        start,
        end,
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
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      console.error("[Overrides] addCrossingSegmentFromDraft error:", e);
      applyError = msg || "Could not snap to road or sidewalk";
      return;
    } finally {
      loading = "";
    }
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
    const appliedOrder = segmentsInLoadedArea;
    const deletedIndex = appliedOrder.findIndex((s) => s.id === id);
    const wasApplied = deletedIndex >= 0 && deletedIndex < appliedCount;
    overrides = { ...overrides, addedCrossings: list };
    await saveOverrides(overrides);
    if (wasApplied && $backend) {
      loading = "Removing crossing";
      await refreshLoadingScreen();
      try {
        // Undo only until we've removed the command for this segment (backend stack order
        // matches appliedOrder). Each undo replays the whole stack, so we do the minimum
        // number of undos to avoid repeated ConnectAllCrossings etc.
        const undosNeeded = appliedCount - deletedIndex;
        for (let i = 0; i < undosNeeded; i++) {
          $backend.editUndo();
          mutationCounter.update((n) => n + 1);
        }
        // Re-apply segments that were after the deleted one (we popped them when we undid).
        const toReapply = appliedOrder.slice(deletedIndex + 1, appliedCount);
        for (const seg of toReapply) {
          if (!isValidSegment(seg)) continue;
          $backend.editAddCrossingSegment(
            seg.start.lng,
            seg.start.lat,
            seg.end.lng,
            seg.end.lat,
            { ...crossingWayTags, ...seg.tags },
          );
          mutationCounter.update((n) => n + 1);
        }
        appliedCount = deletedIndex + toReapply.length;
      } finally {
        loading = "";
      }
    }
  }

  function zoomToSegment(seg: AddedCrossingSegment) {
    if (!isValidSegment(seg)) return;
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

  function exportOverrides() {
    const blob = JSON.stringify(overrides, null, 2);
    downloadGeneratedFile("speedwalk-overrides.json", blob);
  }

  async function deleteAllOverrides() {
    const n = overrides.addedCrossings.length;
    const msg =
      n === 0
        ? "There are no local overrides to delete."
        : `Delete all ${n} local override${n === 1 ? "" : "s"}? This will remove them from storage and from the map. This cannot be undone.`;
    if (!window.confirm(msg)) return;
    if ($backend && appliedCount > 0) {
      loading = "Removing overrides from map";
      await refreshLoadingScreen();
      await unapplyAll();
    }
    overrides = { version: 1, addedCrossings: [] };
    await saveOverrides(overrides);
  }

  let importInput: HTMLInputElement;
  function importOverrides() {
    importInput?.click();
  }

  async function onImportFile(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    const text = await file.text();
    let json: unknown;
    try {
      json = JSON.parse(text);
    } catch (_) {
      applyError = "Invalid JSON";
      input.value = "";
      return;
    }
    const out = manualOverridesSchema.safeParse(json);
    if (!out.success) {
      applyError = "Invalid overrides file format";
      input.value = "";
      return;
    }
    const toMerge = out.data.addedCrossings.map((seg) =>
      seg.id ? seg : { ...seg, id: crypto.randomUUID() },
    );
    overrides = {
      version: 1,
      addedCrossings: [...overrides.addedCrossings, ...toMerge],
    };
    await saveOverrides(overrides);
    input.value = "";

    if (!$backend || toMerge.length === 0) return;
    if (!overridesApplied) return;
    applyError = "";
    try {
      const boundary = JSON.parse($backend.getBoundary());
      const inBoundary = filterSegmentsInBoundary(toMerge, boundary);
      if (inBoundary.length > 0) {
        const prevApplied = appliedCount;
        await applyAll(inBoundary);
        appliedCount = prevApplied + inBoundary.length;
      }
    } catch (_) {}
  }

  const appliedList = $derived(segmentsInLoadedArea.slice(0, appliedCount));
  const notAppliedList = $derived(segmentsInLoadedArea.slice(appliedCount));

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

<Loading {loading} />

<SplitComponent>
  {#snippet left()}
    <Jumbotron
      title="Manual overrides"
      lead="Modify the network by manually removing geometries and adding junctions. Changes are stored in your browser."
    >
      <p class="small mb-2">
        <strong>Add crossing:</strong>
        Place two points on the map (first = left/red, second = right/blue). Drag
        markers to adjust, or click the map to set or move them. If both points are
        off the map, the next click starts a new draft. Use
        <strong>Add crossing</strong>
        or press
        <kbd>a</kbd>
        to snap to the network and save. Use
        <strong>Reset draft</strong>
        to clear both points.
      </p>
      <p class="small mb-2 text-muted">
        The new crossing is a routable segment between the two snapped points on
        the network.
      </p>
      <div class="d-flex gap-2 align-items-center flex-wrap">
        <button
          type="button"
          class="btn btn-outline-secondary btn-sm"
          onclick={() => {
            pointA = null;
            pointB = null;
          }}
          disabled={!pointA && !pointB}
        >
          Reset draft
        </button>
        <button
          type="button"
          class="btn btn-outline-primary btn-sm"
          onclick={() => addCrossingSegmentFromDraft()}
          disabled={!pointA || !pointB || !$backend}
        >
          Add crossing
        </button>
      </div>
    </Jumbotron>

    <FilterNetworkCard />

    {#if !$backend}
      <div class="alert alert-warning py-2 small mb-3" role="alert">
        <strong>Load an area first.</strong>
        Add crossing (two clicks +
        <kbd>a</kbd>
        ), apply, and export/import are available after you load data (relation, polygon,
        or file).
      </div>
    {/if}

    {#if applyError}
      <div
        class="alert alert-danger py-2 small mb-3 d-flex align-items-center justify-content-between"
        role="alert"
      >
        <span>{applyError}</span>
        <button
          type="button"
          class="btn-close btn-close-sm"
          aria-label="Close"
          onclick={() => (applyError = "")}
        ></button>
      </div>
    {/if}

    <CollapsibleCard open={true}>
      {#snippet header()}
        In loaded area: {segmentsInLoadedArea.length} in storage, {appliedCount}
        applied
      {/snippet}
      {#snippet body()}
        {#if segmentsInLoadedArea.length > 0}
          <button
            class="btn mb-3 {overridesApplied ? 'btn-secondary' : 'btn-primary'}"
            onclick={toggleApply}
            disabled={!$backend}
          >
            {overridesApplied
              ? "Unapply manual overrides from current data"
              : "Apply manual overrides to current data"}
          </button>
        {/if}
        {#if notAppliedList.length > 0}
          <h6 class="mt-2">Not applied</h6>
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
        {#if segmentsInLoadedArea.length === 0}
          <p class="text-muted small">
            {#if overrides.addedCrossings.length === 0}
              No manual crossings yet.
            {:else}
              No overrides in loaded area ({overrides.addedCrossings.length} total
              in storage).
            {/if}
          </p>
        {/if}
      {/snippet}
    </CollapsibleCard>

    <div class="mt-3">
      <button
        class="btn btn-secondary btn-sm me-1"
        onclick={exportOverrides}
        disabled={!$backend}
      >
        Export overrides
      </button>
      <button
        class="btn btn-secondary btn-sm me-1"
        onclick={importOverrides}
        disabled={!$backend}
      >
        Import overrides
      </button>
      <button
        class="btn btn-outline-danger btn-sm"
        onclick={() => deleteAllOverrides()}
        disabled={overrides.addedCrossings.length === 0}
      >
        Delete all overrides
      </button>
    </div>
  {/snippet}

  {#snippet main()}
    <MapEvents onclick={onMapClick} />

    <!-- Network and nodes first (bottom), then draft line and points on top so they stay visible -->
    {#if $backend}
      <GeoJSON data={filteredNetworkGeoJSON} generateId>
        <LineLayer
          id="overrides-ways"
          paint={{
            "line-width": roadLineWidth(0),
            "line-color": [
              "case",
              ["==", ["get", "crossing"], "manual"],
              "blue",
              "black",
            ],
          }}
        />
      </GeoJSON>
      <GeoJSON data={filteredNodesGeoJSON} generateId>
        <CircleLayer
          id="overrides-nodes"
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

    {#if pointA && pointB}
      <GeoJSON data={draftLineGeoJSON} generateId>
        <LineLayer
          id="overrides-draft-line"
          paint={{
            "line-width": 4,
            "line-color": "#9b59b6",
            "line-dasharray": [2, 2],
          }}
        />
      </GeoJSON>
    {/if}
    {#if pointA}
      <Marker
        class="overrides-draft-marker overrides-draft-marker--red"
        bind:lngLat={pointA}
        draggable={true}
      >
        {#snippet children()}
          <div
            class="overrides-draft-dot overrides-draft-dot--red"
            title="Left point (drag to move)"
          ></div>
        {/snippet}
      </Marker>
    {/if}
    {#if pointB}
      <Marker
        class="overrides-draft-marker overrides-draft-marker--blue"
        bind:lngLat={pointB}
        draggable={true}
      >
        {#snippet children()}
          <div
            class="overrides-draft-dot overrides-draft-dot--blue"
            title="Right point (drag to move)"
          ></div>
        {/snippet}
      </Marker>
    {/if}

    <Control position="top-right">
      <CollapsibleCard>
        {#snippet header()}Legend{/snippet}
        {#snippet body()}
          <LegendList items={overridesLegendItems} />
        {/snippet}
      </CollapsibleCard>
    </Control>
  {/snippet}
</SplitComponent>

<style>
  :global(.overrides-draft-marker) {
    cursor: grab;
  }
  :global(.overrides-draft-marker:active) {
    cursor: grabbing;
  }
  .overrides-draft-dot {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    border: 2px solid #fff;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
    margin-left: -12px;
    margin-top: -12px;
  }
  .overrides-draft-dot--red {
    background-color: #e74c3c;
  }
  .overrides-draft-dot--blue {
    background-color: #3498db;
  }
</style>
