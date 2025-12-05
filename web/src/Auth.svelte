<script lang="ts">
  import * as OSM from "osm-api";
  import { onMount } from "svelte";
  import { loggedInUser } from "./";

  onMount(handleLogin);

  async function handleLogin() {
    if (OSM.isLoggedIn()) {
      let user = await OSM.getUser("me");
      $loggedInUser = {
        name: user.display_name,
        uid: user.id,
        avatarUrl: user.img?.href || "",
      };
    }
  }

  async function login() {
    try {
      await OSM.login({
        mode: "popup",
        clientId: "vyCV0t-IiskqNBgpiHvuSAmf2nC8K-zfByeFL6XtAzc",
        redirectUrl: `${window.location.origin}/speedwalk/land.html`,
        scopes: ["read_prefs", "write_api"],
      });
      await handleLogin();
    } catch (err) {
      window.alert(`Login failed: ${err}`);
    }
  }

  async function logout() {
    $loggedInUser = undefined;
    await OSM.logout();
  }
</script>

{#if $loggedInUser}
  <div class="nav-item dropdown">
    <button
      class="btn btn-outline-secondary dropdown-toggle"
      data-bs-toggle="dropdown"
      aria-expanded="false"
    >
      {#if $loggedInUser.avatarUrl}
        <img src={$loggedInUser.avatarUrl} alt="OSM avatar" height="30" />
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
  <button class="btn btn-primary" on:click={login}>Login to OSM</button>
{/if}
