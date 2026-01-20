import { writable, type Writable, get } from "svelte/store";

export const settingNamespace = "overpass-ide";
export const defaultServer = "https://overpass-api.de/api/";
// https://wiki.openstreetmap.org/wiki/Overpass_API#Public_Overpass_API_instances
export const suggestedServers = [
  "https://overpass-api.de/api/",
  "https://maps.mail.ru/osm/tools/overpass/api/",
  "https://overpass.openstreetmap.ru/api/",
  "https://overpass.kumi.systems/api/",
];

function getStoredServer(): string {
  if (typeof window === "undefined") {
    return defaultServer;
  }
  const stored = window.localStorage.getItem(settingNamespace);
  return stored || defaultServer;
}

export const overpassServer: Writable<string> = writable(getStoredServer());

// Sync store changes to localStorage
overpassServer.subscribe((server) => {
  if (typeof window !== "undefined") {
    window.localStorage.setItem(settingNamespace, server);
  }
});

export async function fetchOverpass(
  queryOrUrl: string,
): Promise<Response> {
  const isUrl = queryOrUrl.startsWith("http");
  const selectedServer = get(overpassServer);

  const url = isUrl
    ? queryOrUrl
    : `${selectedServer}interpreter`;

  const resp = await fetch(
    url,
    isUrl
      ? undefined
      : {
          method: "POST",
          body: queryOrUrl,
          headers: {
            "Content-Type": "text/plain",
          },
        },
  );

  if (!resp.ok) {
    if (resp.status === 504) {
      throw new Error(
        `Overpass API timed out. The query might be too large. Try again later or switch to a different Overpass server.`,
      );
    }
    throw new Error(
      `Overpass failed: ${resp.status}. Try again or switch to a different Overpass server.`,
    );
  }

  return resp;
}
