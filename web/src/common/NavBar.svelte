<script lang="ts">
  import Auth from "./Auth.svelte";
  import logo from "../../assets/logo.svg?url";
  import { type Mode, mode, backend } from "../";
  import { Modal } from "svelte-utils";

  let showInfo = false;

  let mainActions = [
    [{ kind: "sidewalks" }, "Sidewalks"],
    [{ kind: "crossings" }, "Crossings"],
    [{ kind: "disconnections" }, "Network disconnections"],
  ] as [Mode, string][];

  let problemActions = [
    [
      { kind: "node_problems", problem: "missing crossing node" },
      "Missing crossing nodes",
    ],
    [
      {
        kind: "node_problems",
        problem: "separate sidewalks should be continued here",
      },
      "Sidewalks that end",
    ],
    [
      { kind: "way_problems", problem: "missing footway=crossing" },
      "Footways that should be crossings",
    ],
    [
      {
        kind: "way_problems",
        problem: "possible separate sidewalk near way without it tagged",
      },
      "Separate sidewalks not tagged",
    ],
    [
      {
        kind: "way_problems",
        problem: "sidewalk=separate is ambiguous about the side",
      },
      "Ambiguous sidewalk=separate",
    ],
    [
      {
        kind: "way_problems",
        problem:
          "sidewalk:left and sidewalk:right should each be tagged as separate or no",
      },
      "Inconsistent tagging on each side",
    ],
  ] as [Mode, string][];
</script>

<ul class="nav nav-underline">
  <img src={logo} style="height: 30px" alt="A/B Street logo" />
  <h3>Speedwalk</h3>

  {#if $backend}
    {#each mainActions as [setMode, label]}
      <li class="nav-item">
        <!-- svelte-ignore a11y-invalid-attribute -->
        <a
          class="nav-link"
          href="#"
          on:click={() => ($mode = setMode)}
          class:active={JSON.stringify($mode) == JSON.stringify(setMode)}
        >
          {label}
        </a>
      </li>
    {/each}

    <li class="nav-item dropdown">
      <!-- svelte-ignore a11y-invalid-attribute -->
      <a
        class="nav-link dropdown-toggle"
        data-bs-toggle="dropdown"
        href="#"
        role="button"
        aria-expanded="false"
      >
        Fix problems
      </a>
      <ul class="dropdown-menu">
        {#each problemActions as [setMode, label]}
          <li>
            <!-- svelte-ignore a11y-invalid-attribute -->
            <a
              class="dropdown-item"
              href="#"
              on:click={() => ($mode = setMode)}
            >
              {label}
            </a>
          </li>
        {/each}
      </ul>
    </li>
  {/if}

  <li class="nav-item ms-auto">
    <!-- svelte-ignore a11y-invalid-attribute -->
    <a class="nav-link" href="#" on:click={() => (showInfo = true)}>
      <i class="fa-solid fa-circle-info"></i>
      About
    </a>
  </li>

  <Auth />
</ul>

<Modal bind:show={showInfo}>
  <h1>Welcome to Speedwalk</h1>

  <p>
    This tool helps you quickly assess how sidewalks are mapped in OSM. You can
    find and fix common tagging problems. The tool assumes you understand the
    correct <a
      href="https://wiki.openstreetmap.org/wiki/Sidewalks"
      target="_blank"
    >
      sidewalk mapping conventions
    </a>
    . If you are unsure about some edits you make, you can download the changeset
    file and check in JOSM, rather than uploading directly.
  </p>

  <p>
    This is an <a
      href="https://github.com/a-b-street/speedwalk"
      target="_blank"
    >
      open source project
    </a>
    developed without funding by
    <a href="https://abstreet.uk" target="_blank">A/B Street Ltd</a>
    . Please file a Github issue or contact
    <a href="mailto:dustin@abstreet.uk" target="_blank">dustin@abstreet.uk</a>
    with feedback.
    <b>This is an alpha tool; there will be problems!</b>
  </p>

  <button class="btn btn-primary" on:click={() => (showInfo = false)}>
    Start
  </button>
</Modal>
