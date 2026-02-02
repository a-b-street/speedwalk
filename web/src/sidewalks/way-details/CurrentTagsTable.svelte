<script lang="ts">
  let { tags }: { tags: Record<string, string> } = $props();

  function getSortedTags(
    tagRecord: Record<string, string>,
  ): Array<[string, string]> {
    const entries = Object.entries(tagRecord);
    const sidewalkTags = entries.filter(([key]) =>
      key.toLowerCase().startsWith("sidewalk"),
    );
    const otherTags = entries.filter(
      ([key]) => !key.toLowerCase().startsWith("sidewalk"),
    );
    sidewalkTags.sort(([a], [b]) => a.localeCompare(b));
    otherTags.sort(([a], [b]) => a.localeCompare(b));
    return [...sidewalkTags, ...otherTags] as Array<[string, string]>;
  }
</script>

<table class="table table-bordered">
  <thead>
    <tr>
      <th>Key</th>
      <th>Value</th>
    </tr>
  </thead>
  <tbody>
    {#each getSortedTags(tags) as [key, value]}
      <tr class:table-active={key.toLowerCase().includes("sidewalk")}>
        <td>{key}</td>
        <td>{value}</td>
      </tr>
    {/each}
  </tbody>
</table>
