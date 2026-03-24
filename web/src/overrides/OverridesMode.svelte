<script lang="ts">
  import {
    getOverrides,
    saveOverrides,
    filterSegmentsInBoundary,
    filterDeletionsInBoundary,
  } from "../common/localOverrides";
  import {
    crossingStableId,
    deletionStableId,
    inViewIdSets,
    type ViewportBounds,
  } from "../common/viewportOverrides";
  import {
    type AddedCrossingSegment,
    type DeletedWaySegment,
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
  import type { FeatureCollection, LineString, Point } from "geojson";
  import { type NodeProps } from "../sidewalks";
  import { roadLineWidth } from "../sidewalks";
  import { MAPILLARY_PIN_LAYER_IDS_LIST } from "../common/mapillaryLayers";
  import { overridesLegendColors, overridesLegendItems } from "./legend";

  /** First click: red dot (start of crossing segment). Second click: blue dot (end). */
  let pointA: { lng: number; lat: number } | null = $state(null);
  let pointB: { lng: number; lat: number } | null = $state(null);
  let loading = $state("");
  let applyError = $state("");
  let overrides: ManualOverrides = $state({
    version: 1,
    addedCrossings: [],
    deletedWaySegments: [],
  });
  let overridesApplied = $state(true);
  /** Backend command stack: all crossings in order, then all manual deletions in order. */
  let appliedCrossingCount = $state(0);
  let appliedDeletionCount = $state(0);
  let nodes: FeatureCollection<Point, NodeProps> = $state.raw({
    type: "FeatureCollection",
    features: [],
  });

  /** Current map camera; updated on pan/zoom for in-view row indicators. */
  let viewportBounds = $state<ViewportBounds | null>(null);

  $effect(() => {
    const mapInstance = $map;
    if (!mapInstance) {
      viewportBounds = null;
      return;
    }
    const mapRef = mapInstance;
    function syncViewport() {
      const b = mapRef.getBounds();
      viewportBounds = {
        west: b.getWest(),
        south: b.getSouth(),
        east: b.getEast(),
        north: b.getNorth(),
      };
    }
    syncViewport();
    mapRef.on("moveend", syncViewport);
    mapRef.on("zoomend", syncViewport);
    return () => {
      mapRef.off("moveend", syncViewport);
      mapRef.off("zoomend", syncViewport);
    };
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
      appliedCrossingCount = 0;
      appliedDeletionCount = 0;
      return;
    }
    getOverrides().then((data) => {
      overrides = data;
      const boundary = JSON.parse(b.getBoundary());
      const list = filterSegmentsInBoundary(data.addedCrossings, boundary);
      const delList = filterDeletionsInBoundary(
        data.deletedWaySegments,
        boundary,
      );
      if (
        overridesApplied &&
        appliedCrossingCount === 0 &&
        appliedDeletionCount === 0 &&
        (list.length > 0 || delList.length > 0)
      ) {
        applyEverythingInBoundary(list, delList);
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

  const deletionsInLoadedArea = $derived.by(() => {
    if (!$backend) return [];
    try {
      const boundary = JSON.parse($backend.getBoundary());
      return filterDeletionsInBoundary(overrides.deletedWaySegments, boundary);
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

  /** Draw manual deletions using their node pairs as thin dotted red line segments. */
  const deletedWaysGeoJSON = $derived.by(() => {
    const idToCoord = new Map<number, [number, number]>();
    for (const f of nodes.features) {
      const p = f.properties as { id?: number } | undefined;
      if (p?.id == null) continue;
      const coords = f.geometry?.coordinates;
      if (!coords || coords.length < 2) continue;
      idToCoord.set(p.id, [coords[0], coords[1]]);
    }

    const features = deletionsInLoadedArea.flatMap((seg) => {
      const c1 = idToCoord.get(seg.node1);
      const c2 = idToCoord.get(seg.node2);
      if (!c1 || !c2) return [];
      return [
        {
          type: "Feature" as const,
          geometry: {
            type: "LineString" as const,
            coordinates: [c1, c2],
          },
          properties: {
            wayId: seg.wayId,
            node1: seg.node1,
            node2: seg.node2,
          },
        },
      ];
    });

    return {
      type: "FeatureCollection" as const,
      features,
    } as FeatureCollection<LineString>;
  });

  const crossingWayTags = {
    highway: "footway",
    footway: "crossing",
    crossing: "manual",
  };

  /** Apply crossings first, then manual deletions (matches backend undo order). */
  async function applyEverythingInBoundary(
    crossings: AddedCrossingSegment[],
    deletions: DeletedWaySegment[],
  ) {
    if (!$backend) return;
    applyError = "";
    loading = "Applying overrides";
    await refreshLoadingScreen();
    try {
      for (const seg of crossings) {
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
      appliedCrossingCount = crossings.length;
      for (const d of deletions) {
        $backend.editManualDeleteEdge(
          BigInt(d.wayId),
          BigInt(d.node1),
          BigInt(d.node2),
        );
        mutationCounter.update((n) => n + 1);
      }
      appliedDeletionCount = deletions.length;
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
    if (!$backend || (appliedCrossingCount === 0 && appliedDeletionCount === 0))
      return;
    loading = "Unapplying overrides";
    await refreshLoadingScreen();
    try {
      for (let i = 0; i < appliedDeletionCount; i++) {
        $backend.editUndo();
        mutationCounter.update((n) => n + 1);
      }
      for (let i = 0; i < appliedCrossingCount; i++) {
        $backend.editUndo();
        mutationCounter.update((n) => n + 1);
      }
      appliedDeletionCount = 0;
      appliedCrossingCount = 0;
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
      await applyEverythingInBoundary(
        segmentsInLoadedArea,
        deletionsInLoadedArea,
      );
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
      appliedCrossingCount++;
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
    if (
      e.target instanceof HTMLInputElement ||
      e.target instanceof HTMLTextAreaElement
    )
      return;
    if (e.key === "a") {
      addCrossingSegmentFromDraft();
      e.preventDefault();
    } else if (e.key === "d") {
      deleteSegmentFromDraft();
      e.preventDefault();
    }
  }

  type ResolvedDeletionJson = {
    edges: Array<{
      way_id: number;
      node1: number;
      node2: number;
      mid_lat: number;
      mid_lng: number;
      tags: Record<string, string>;
    }>;
  };

  async function deleteSegmentFromDraft() {
    if (!pointA || !pointB || !$backend) return;
    loading = "Resolving manual deletion";
    await refreshLoadingScreen();
    try {
      const raw = JSON.parse(
        $backend.resolveManualDeletion(
          pointA.lng,
          pointA.lat,
          pointB.lng,
          pointB.lat,
        ),
      ) as ResolvedDeletionJson;
      const edges = raw.edges ?? [];
      if (edges.length === 0) {
        applyError = "No segment found to delete.";
        return;
      }
      const newEntries: DeletedWaySegment[] = edges.map((e) => ({
        id: crypto.randomUUID(),
        wayId: e.way_id,
        node1: e.node1,
        node2: e.node2,
        midLat: e.mid_lat,
        midLng: e.mid_lng,
        tags: e.tags ?? {},
      }));
      overrides = {
        ...overrides,
        deletedWaySegments: [...overrides.deletedWaySegments, ...newEntries],
      };
      await saveOverrides(overrides);
      if (overridesApplied) {
        for (const e of newEntries) {
          $backend.editManualDeleteEdge(
            BigInt(e.wayId),
            BigInt(e.node1),
            BigInt(e.node2),
          );
          mutationCounter.update((n) => n + 1);
        }
        appliedDeletionCount += newEntries.length;
      }
      applyError = "";
      pointA = null;
      pointB = null;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      applyError = msg || "Could not delete segment";
    } finally {
      loading = "";
    }
  }

  async function removeAddedCrossing(segment: AddedCrossingSegment) {
    if (!$backend) return;
    const id = segment.id;
    const list = overrides.addedCrossings.filter((s) => s.id !== id);
    const appliedOrder = segmentsInLoadedArea;
    const deletedIndex = appliedOrder.findIndex((s) => s.id === id);
    const wasApplied = deletedIndex >= 0 && deletedIndex < appliedCrossingCount;
    overrides = { ...overrides, addedCrossings: list };
    await saveOverrides(overrides);
    if (wasApplied && $backend) {
      loading = "Removing crossing";
      await refreshLoadingScreen();
      try {
        // Undo only until we've removed the command for this segment (backend stack order
        // matches appliedOrder). Each undo replays the whole stack, so we do the minimum
        // number of undos to avoid repeated ConnectAllCrossings etc.
        const undosNeeded = appliedCrossingCount - deletedIndex;
        for (let i = 0; i < undosNeeded; i++) {
          $backend.editUndo();
          mutationCounter.update((n) => n + 1);
        }
        // Re-apply segments that were after the deleted one (we popped them when we undid).
        const toReapply = appliedOrder.slice(
          deletedIndex + 1,
          appliedCrossingCount,
        );
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
        appliedCrossingCount = deletedIndex + toReapply.length;
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

  function zoomToDeletion(seg: DeletedWaySegment) {
    const pad = 0.0001;
    $map?.fitBounds(
      [
        [seg.midLng - pad, seg.midLat - pad],
        [seg.midLng + pad, seg.midLat + pad],
      ],
      { padding: 50, maxZoom: 18 },
    );
  }

  function exportOverrides() {
    const prefixedDeleted = overrides.deletedWaySegments.map((seg) => ({
      ...seg,
      tags: Object.fromEntries(
        Object.entries(seg.tags).map(([k, v]) => [`manually_deleted:${k}`, v]),
      ),
    }));
    const payload: ManualOverrides = {
      ...overrides,
      deletedWaySegments: prefixedDeleted,
    };
    const blob = JSON.stringify(payload, null, 2);
    downloadGeneratedFile("speedwalk-overrides.json", blob);
  }

  async function deleteAllOverrides() {
    const n =
      overrides.addedCrossings.length + overrides.deletedWaySegments.length;
    const msg =
      n === 0
        ? "There are no local overrides to delete."
        : `Delete all ${n} local override${n === 1 ? "" : "s"}? This will remove them from storage and from the map. This cannot be undone.`;
    if (!window.confirm(msg)) return;
    if ($backend && (appliedCrossingCount > 0 || appliedDeletionCount > 0)) {
      loading = "Removing overrides from map";
      await refreshLoadingScreen();
      await unapplyAll();
    }
    overrides = {
      version: 1,
      addedCrossings: [],
      deletedWaySegments: [],
    };
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
    const toMergeDel = (out.data.deletedWaySegments ?? []).map((seg) =>
      seg.id ? seg : { ...seg, id: crypto.randomUUID() },
    );
    const normalizedDel = toMergeDel.map((seg) => ({
      ...seg,
      tags: Object.fromEntries(
        Object.entries(seg.tags).map(([k, v]) => {
          if (k.startsWith("manually_deleted:")) {
            return [k.slice("manually_deleted:".length), v];
          }
          return [k, v];
        }),
      ),
    }));
    overrides = {
      version: out.data.version ?? 1,
      addedCrossings: [...overrides.addedCrossings, ...toMerge],
      deletedWaySegments: [...overrides.deletedWaySegments, ...normalizedDel],
    };
    await saveOverrides(overrides);
    input.value = "";

    if (!$backend) return;
    if (!overridesApplied) return;
    applyError = "";
    try {
      const boundary = JSON.parse($backend.getBoundary());
      const inBoundary = filterSegmentsInBoundary(toMerge, boundary);
      const inBoundaryDel = filterDeletionsInBoundary(normalizedDel, boundary);
      if (inBoundary.length > 0) {
        const prevCross = appliedCrossingCount;
        loading = "Applying imported crossings";
        await refreshLoadingScreen();
        try {
          for (const seg of inBoundary) {
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
          appliedCrossingCount = prevCross + inBoundary.length;
        } finally {
          loading = "";
        }
      }
      if (inBoundaryDel.length > 0) {
        const prevDel = appliedDeletionCount;
        loading = "Applying imported deletions";
        await refreshLoadingScreen();
        try {
          for (const d of inBoundaryDel) {
            $backend.editManualDeleteEdge(
              BigInt(d.wayId),
              BigInt(d.node1),
              BigInt(d.node2),
            );
            mutationCounter.update((n) => n + 1);
          }
          appliedDeletionCount = prevDel + inBoundaryDel.length;
        } finally {
          loading = "";
        }
      }
    } catch (_) {}
  }

  async function removeDeletedSegment(segment: DeletedWaySegment) {
    if (!$backend) return;
    const id = segment.id;
    const list = overrides.deletedWaySegments.filter((s) => s.id !== id);
    const appliedOrder = deletionsInLoadedArea;
    const deletedIndex = appliedOrder.findIndex((s) => s.id === id);
    const wasApplied = deletedIndex >= 0 && deletedIndex < appliedDeletionCount;
    overrides = { ...overrides, deletedWaySegments: list };
    await saveOverrides(overrides);
    if (wasApplied && $backend) {
      loading = "Removing manual deletion";
      await refreshLoadingScreen();
      try {
        const undosNeeded = appliedDeletionCount - deletedIndex;
        for (let i = 0; i < undosNeeded; i++) {
          $backend.editUndo();
          mutationCounter.update((n) => n + 1);
        }
        const toReapply = appliedOrder.slice(
          deletedIndex + 1,
          appliedDeletionCount,
        );
        for (const d of toReapply) {
          $backend.editManualDeleteEdge(
            BigInt(d.wayId),
            BigInt(d.node1),
            BigInt(d.node2),
          );
          mutationCounter.update((n) => n + 1);
        }
        appliedDeletionCount = deletedIndex + toReapply.length;
      } finally {
        loading = "";
      }
    }
  }

  const appliedCrossingList = $derived(
    segmentsInLoadedArea.slice(0, appliedCrossingCount),
  );
  const notAppliedCrossingList = $derived(
    segmentsInLoadedArea.slice(appliedCrossingCount),
  );
  const appliedDeletionList = $derived(
    deletionsInLoadedArea.slice(0, appliedDeletionCount),
  );
  const notAppliedDeletionList = $derived(
    deletionsInLoadedArea.slice(appliedDeletionCount),
  );

  const inViewSets = $derived.by(() => {
    if (!viewportBounds) return null;
    return inViewIdSets(
      segmentsInLoadedArea,
      deletionsInLoadedArea,
      viewportBounds,
    );
  });

  function crossingRowHighlightClass(seg: AddedCrossingSegment): string {
    if (!inViewSets) return "";
    return inViewSets.crossingIds.has(crossingStableId(seg))
      ? "ps-2 border-start border-3 border-primary rounded-start"
      : "";
  }

  function deletionRowHighlightClass(seg: DeletedWaySegment): string {
    if (!inViewSets) return "";
    return inViewSets.deletionIds.has(deletionStableId(seg))
      ? "ps-2 border-start border-3 border-primary rounded-start"
      : "";
  }

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
      <p class="small mb-2">
        <strong>Delete segment:</strong>
        Use the same two points on one road or path, then
        <strong>Delete segment</strong>
        or press
        <kbd>d</kbd>
        to remove those road edges from the routable network (they disappear from
        the map here).
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
        <button
          type="button"
          class="btn btn-outline-danger btn-sm"
          onclick={() => deleteSegmentFromDraft()}
          disabled={!pointA || !pointB || !$backend}
        >
          Delete segment
        </button>
      </div>
    </Jumbotron>

    <FilterNetworkCard />

    {#if !$backend}
      <div class="alert alert-warning py-2 small mb-3" role="alert">
        <strong>Load an area first.</strong>
        Add crossing (
        <kbd>a</kbd>
        ) or delete segment (
        <kbd>d</kbd>
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
        In loaded area: {segmentsInLoadedArea.length} crossing(s), {deletionsInLoadedArea.length}
        deletion(s) in storage; {appliedCrossingCount}+{appliedDeletionCount} applied
        · In current view: {inViewSets?.crossingIds.size ?? 0} crossing(s), {inViewSets
          ?.deletionIds.size ?? 0} deletion(s)
      {/snippet}
      {#snippet body()}
        {#if segmentsInLoadedArea.length > 0 || deletionsInLoadedArea.length > 0}
          <button
            class="btn mb-3 {overridesApplied
              ? 'btn-secondary'
              : 'btn-primary'}"
            onclick={toggleApply}
            disabled={!$backend}
          >
            {overridesApplied
              ? "Unapply manual overrides from current data"
              : "Apply manual overrides to current data"}
          </button>
        {/if}
        {#if notAppliedCrossingList.length > 0}
          <h6 class="mt-2">Crossings — not applied</h6>
          <ul class="list-unstyled small">
            {#each notAppliedCrossingList as seg}
              <li
                class="d-flex align-items-center gap-2 mb-1 {crossingRowHighlightClass(
                  seg,
                )}"
              >
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
        {#if appliedCrossingList.length > 0}
          <h6 class="mt-2">Crossings — applied</h6>
          <ul class="list-unstyled small">
            {#each appliedCrossingList as seg}
              <li
                class="d-flex align-items-center gap-2 mb-1 {crossingRowHighlightClass(
                  seg,
                )}"
              >
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
        {#if notAppliedDeletionList.length > 0}
          <h6 class="mt-2">Deletions — not applied</h6>
          <ul class="list-unstyled small">
            {#each notAppliedDeletionList as seg}
              <li
                class="d-flex align-items-center gap-2 mb-1 {deletionRowHighlightClass(
                  seg,
                )}"
              >
                <span class="text-break small">
                  way {seg.wayId} nodes {seg.node1}–{seg.node2}
                </span>
                <button
                  type="button"
                  class="btn btn-link p-0 small text-primary"
                  onclick={() => zoomToDeletion(seg)}
                >
                  Zoom
                </button>
                <button
                  class="btn btn-sm btn-outline-danger"
                  onclick={() => removeDeletedSegment(seg)}
                >
                  Remove
                </button>
              </li>
            {/each}
          </ul>
        {/if}
        {#if appliedDeletionList.length > 0}
          <h6 class="mt-2">Deletions — applied</h6>
          <ul class="list-unstyled small">
            {#each appliedDeletionList as seg}
              <li
                class="d-flex align-items-center gap-2 mb-1 {deletionRowHighlightClass(
                  seg,
                )}"
              >
                <span class="text-break small">
                  way {seg.wayId} nodes {seg.node1}–{seg.node2}
                </span>
                <button
                  type="button"
                  class="btn btn-link p-0 small text-primary"
                  onclick={() => zoomToDeletion(seg)}
                >
                  Zoom
                </button>
                <button
                  class="btn btn-sm btn-outline-danger"
                  onclick={() => removeDeletedSegment(seg)}
                >
                  Remove
                </button>
              </li>
            {/each}
          </ul>
        {/if}
        {#if segmentsInLoadedArea.length === 0 && deletionsInLoadedArea.length === 0}
          <p class="text-muted small">
            {#if overrides.addedCrossings.length === 0 && overrides.deletedWaySegments.length === 0}
              No manual crossings or deletions yet.
            {:else}
              No overrides in loaded area ({overrides.addedCrossings.length} crossing(s),
              {overrides.deletedWaySegments.length}
              deletion(s) total in storage).
            {/if}
          </p>
        {/if}
      {/snippet}
    </CollapsibleCard>

    <div class="mt-3">
      <button
        class="btn btn-secondary btn-sm me-1"
        onclick={exportOverrides}
        disabled={!$backend ||
          (overrides.addedCrossings.length === 0 &&
            overrides.deletedWaySegments.length === 0)}
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
        disabled={overrides.addedCrossings.length === 0 &&
          overrides.deletedWaySegments.length === 0}
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
              overridesLegendColors["Manually added crossing"],
              overridesLegendColors["Base data"],
            ],
          }}
        />
      </GeoJSON>
      <GeoJSON data={deletedWaysGeoJSON} generateId>
        <LineLayer
          id="overrides-deleted-ways"
          paint={{
            "line-width": 1,
            "line-color": overridesLegendColors["Manually deleted way"],
            "line-dasharray": [1, 2],
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
              overridesLegendColors["Manually added crossing"],
              overridesLegendColors["Base data"],
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
