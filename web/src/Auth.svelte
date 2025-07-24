<script lang="ts">
  import * as OSM from "osm-api";
  import { onMount } from "svelte";

  let loggedInUser: { name: string; uid: number; avatarUrl: string } | null =
    null;

  onMount(async () => {
    await OSM.authReady;
    if (OSM.isLoggedIn()) {
      let user = await OSM.getUser("me");
      loggedInUser = {
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
      redirectUrl: "http://127.0.0.1:5173/speedwalk/index.html",
      scopes: ["read_prefs", "write_api"],
    });
  }

  async function logout() {
    loggedInUser = null;
    await OSM.logout();
  }
</script>

{#if loggedInUser}
  <p>Logged in as {loggedInUser.name} (id {loggedInUser.uid})</p>
  {#if loggedInUser.avatarUrl}
    <img src={loggedInUser.avatarUrl} alt="OSM avatar" />
  {/if}

  <button class="btn btn-danger" on:click={logout}>Logout</button>
{:else}
  <button class="btn btn-primary" on:click={login}>Login</button>
{/if}
