<script lang="ts">
  let {
    tags,
    recentlyAddedTags = new Set<string>(),
  }: {
    tags: Record<string, string>;
    recentlyAddedTags?: Set<string>;
  } = $props();

  // Order is important - tags are sorted by this order
  const MAIN_TAG_PREFIXES = [
    "sidewalk",
    "highway",
    "footway",
    "path",
    "cycleway",
    "crossing",
    "is_sidepath",
    "foot",
    "bicycle",
  ];

  function isMainTag(key: string): boolean {
    const lowerKey = key.toLowerCase();
    return MAIN_TAG_PREFIXES.some((prefix) => lowerKey.startsWith(prefix));
  }

  function getSortedTags(
    tagRecord: Record<string, string>,
  ): Array<[string, string]> {
    const entries = Object.entries(tagRecord);
    function getPrefixOrder(key: string): number {
      const lowerKey = key.toLowerCase();
      const index = MAIN_TAG_PREFIXES.findIndex((prefix) =>
        lowerKey.startsWith(prefix),
      );
      return index === -1 ? MAIN_TAG_PREFIXES.length : index;
    }
    return entries.sort(([a], [b]) => {
      const orderA = getPrefixOrder(a);
      const orderB = getPrefixOrder(b);
      if (orderA !== orderB) {
        return orderA - orderB;
      }
      return a.localeCompare(b);
    }) as Array<[string, string]>;
  }
</script>

<table class="table table-bordered table-sm mt-3 tag-table-group">
  <thead class="table-light">
    <tr>
      <th>Key</th>
      <th>Value</th>
    </tr>
  </thead>
  <tbody>
    {#each getSortedTags(tags) as [key, value]}
      {@const isRecent = recentlyAddedTags.has(key)}
      <tr>
        <td
          class:tag-muted={!isMainTag(key)}
          class:tag-recent={isRecent}
        >
          {key}
        </td>
        <td
          class:tag-muted={!isMainTag(key)}
          class:tag-recent={isRecent}
        >
          {value}
        </td>
      </tr>
    {/each}
  </tbody>
</table>

<style>
  .tag-table-group .tag-muted {
    color: #adb5bd;
  }
  .tag-table-group:hover .tag-muted {
    color: inherit;
  }
  .tag-table-group .tag-recent {
    font-weight: bold;
  }
</style>
