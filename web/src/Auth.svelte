<script lang="ts">
  // @ts-expect-error No types
  import { osmAuth } from "osm-auth";

  let auth = osmAuth({
    client_id: "vyCV0t-IiskqNBgpiHvuSAmf2nC8K-zfByeFL6XtAzc",
    redirect_uri: "http://127.0.0.1:5173/speedwalk/index.html",
    scope: "read_prefs write_api",
    singlepage: true,
  });

  let loggedInUser: { name: string; uid: number; avatarUrl: string } | null =
    null;
  if (
    window.location.search
      .slice(1)
      .split("&")
      .some((p) => p.startsWith("code="))
  ) {
    login();
  }

  // TODO Try async everything style if this works
  function login() {
    auth.authenticate((err: any, result: any) => {
      if (err) {
        window.alert(err);
        return;
      }

      auth.xhr(
        { method: "GET", path: "/api/0.6/user/details" },
        (err: any, result: any) => {
          if (result) {
            history.pushState({}, "", window.location.pathname);
            let user = result.getElementsByTagName("user")[0];
            let avatar = result.getElementsByTagName("img")[0];
            loggedInUser = {
              name: user.getAttribute("display_name"),
              uid: user.getAttribute("id"),
              avatarUrl: avatar?.getAttribute("href") || "",
            };
          } else if (err) {
            window.alert(err);
          }
        },
      );
    });
  }

  function logout() {
    loggedInUser = null;
    auth.logout();
  }
</script>

{#if loggedInUser}
  <p>Logged in as {loggedInUser.name} (id {loggedInUser.uid})</p>
  {#if loggedInUser.avatarUrl}
    <img src={loggedInUser.avatarUrl} alt="OSM avatar" />
  {/if}

  <button on:click={logout}>Logout</button>
{:else}
  <button on:click={login}>Login</button>
{/if}
