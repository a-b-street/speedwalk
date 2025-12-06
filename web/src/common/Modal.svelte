<script lang="ts">
  export let show: boolean;
  export let closeable = true;

  // TODO https://caniuse.com/wf-dialog-closedby not supported yet

  // Relies on external styling
  let modalDialog: HTMLDialogElement | undefined = undefined;

  $: {
    if (modalDialog) {
      if (show) {
        modalDialog.showModal();
      } else {
        modalDialog.close();
      }
    }
  }

  function onClick(e: MouseEvent) {
    // only dismiss the modal when clicking outside of the inner dialog content, on the dialog itself.
    if (e.target == modalDialog) {
      e.stopPropagation();
      if (closeable) {
        show = false;
      }
    }
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.key == "Escape" || e.key == "Enter") {
      e.stopPropagation();
      if (closeable) {
        show = false;
      }
    }
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<dialog
  bind:this={modalDialog}
  on:click={onClick}
  on:keydown={onKeyDown}
  closedby="none"
>
  <div><slot /></div>
</dialog>

<style>
  div {
    max-width: 80vw;
    max-height: 80vh;
  }

  dialog::backdrop {
    backdrop-filter: blur(2px);
  }
</style>
