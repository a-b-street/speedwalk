<script lang="ts">
  export let list: any[];
  export let idx = 0;

  function prev() {
    if (idx != 0) {
      idx--;
    }
  }

  function next() {
    if (idx != list.length - 1) {
      idx++;
    }
  }
</script>

<div
  style="display: flex; justify-content: space-between; align-items: center;"
>
  <button disabled={idx == 0} on:click={prev}>Previous</button>
  {idx + 1} / {list.length}
  <button disabled={idx == list.length - 1} on:click={next}>Next</button>
</div>
