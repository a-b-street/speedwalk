<script lang="ts">
  import * as OSM from "osm-api";
  import { onMount } from "svelte";
  import { loggedInUser } from "./";

  onMount(async () => {
    await OSM.authReady;
    if (OSM.isLoggedIn()) {
      let user = await OSM.getUser("me");
      $loggedInUser = {
        name: user.display_name,
        uid: user.id,
        avatarUrl: user.img?.href || "",
      };
      history.pushState({}, "", window.location.pathname);
    }
  });

  function login() {
    OSM.login({
      mode: "redirect",
      clientId: "vyCV0t-IiskqNBgpiHvuSAmf2nC8K-zfByeFL6XtAzc",
      redirectUrl: "http://127.0.0.1:5174/speedwalk/index.html",
      scopes: ["read_prefs", "write_api"],
    });
  }

  async function logout() {
    $loggedInUser = undefined;
    await OSM.logout();
  }
</script>

{#if $loggedInUser}
  <div class="dropdown mb-3">
    <button
      class="btn btn-outline-secondary dropdown-toggle"
      data-bs-toggle="dropdown"
      aria-expanded="false"
    >
      {#if $loggedInUser.avatarUrl}
        <img src={$loggedInUser.avatarUrl} alt="OSM avatar" />
      {/if}
      {$loggedInUser.name}
    </button>
    <ul class="dropdown-menu">
      <li>
        <!-- svelte-ignore a11y-invalid-attribute -->
        <a class="dropdown-item" href="#" on:click={logout}>Logout</a>
      </li>
    </ul>
  </div>
{:else}
  <button class="btn btn-primary mb-3" on:click={login}>Login</button>
{/if}
