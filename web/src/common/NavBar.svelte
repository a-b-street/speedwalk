<script lang="ts">
  import Auth from "./Auth.svelte";
  import logo from "../../assets/logo.svg?url";
  import {
    type Mode,
    type UseCase,
    mode,
    backend,
    useCase,
    loggedInUser,
    ROUTE_NETWORK_ONLY_MODE_KINDS,
    DEFAULT_AUDIT_MODE,
  } from "../";
  import { Modal, LocalStorageWrapper } from "svelte-utils";
  import LoadAnotherArea from "./LoadAnotherArea.svelte";

  let showInfo = $state(false);

  const auditActions = [
    [{ kind: "sidewalks" }, "Sidewalks"],
    [{ kind: "crossings" }, "Crossings"],
    [{ kind: "disconnections" }, "Disconnections"],
  ] as [Mode, string][];

  const routeNetworkActions = [
    ...auditActions,
    [{ kind: "generator" }, "Generator"],
    [{ kind: "overrides" }, "Overrides"],
    [{ kind: "export" }, "Export"],
  ] as [Mode, string][];

  const useCaseOptions: { value: UseCase; label: string }[] = [
    { value: "audit", label: "audit and map sidewalks" },
    { value: "route-networks", label: "generate and export route networks" },
  ];

  let mainActions = $derived(
    $useCase === "route-networks" ? routeNetworkActions : auditActions,
  );
  let currentUseCaseLabel = $derived(
    useCaseOptions.find((o) => o.value === $useCase)?.label ??
      useCaseOptions[0].label,
  );
</script>

<ul class="nav nav-underline">
  <img src={logo} style="height: 30px" alt="A/B Street logo" />
  <h3>Speedwalk</h3>

  {#if $backend}
    <LoadAnotherArea />

    {#if !$loggedInUser}
      <li class="nav-item dropdown">
        <button
          class="btn btn-link nav-link dropdown-toggle text-body text-decoration-none py-0"
          data-bs-toggle="dropdown"
          aria-expanded="false"
          type="button"
        >
          <LocalStorageWrapper>
            <span>I want to {currentUseCaseLabel}</span>
          </LocalStorageWrapper>
        </button>
        <ul class="dropdown-menu">
          {#each useCaseOptions as opt}
            <li>
              <button
                class="dropdown-item"
                type="button"
                class:active={$useCase === opt.value}
                onclick={() => {
                  useCase.set(opt.value);
                  if (
                    opt.value === "audit" &&
                    ROUTE_NETWORK_ONLY_MODE_KINDS.includes($mode.kind)
                  ) {
                    mode.set(DEFAULT_AUDIT_MODE);
                  }
                }}
              >
                I want to {opt.label}
              </button>
            </li>
          {/each}
        </ul>
      </li>
    {/if}

    {#each mainActions as [setMode, label]}
      <li class="nav-item">
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          class="nav-link"
          href="#"
          onclick={() => ($mode = setMode)}
          class:active={JSON.stringify($mode) == JSON.stringify(setMode)}
        >
          {label}
        </a>
      </li>
    {/each}
  {/if}

  <li class="nav-item ms-auto">
    <!-- svelte-ignore a11y_invalid_attribute -->
    <a class="nav-link" href="#" onclick={() => (showInfo = true)}>
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
    developed by
    <a href="https://abstreet.uk" target="_blank">A/B Street Ltd</a>
    and
    <a href="https://fixmycity.de/" target="_blank">FixMyCity GmbH</a>
    , partly funded by FixMyCity. Please
    <a href="https://github.com/a-b-street/speedwalk/issues/" target="_blank">
      file a Github issue
    </a>
    or contact
    <a href="mailto:dustin@abstreet.uk" target="_blank">dustin@abstreet.uk</a>
    with feedback.
    <b>This is an alpha tool; there will be problems!</b>
  </p>

  <button class="btn btn-primary" onclick={() => (showInfo = false)}>
    Start
  </button>
</Modal>
