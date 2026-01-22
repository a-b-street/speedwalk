export async function fetchOverpass(queryOrUrl: string): Promise<Response> {
  const isUrl = queryOrUrl.startsWith("http");
  const url = isUrl ? queryOrUrl : `https://overpass-api.de/api/interpreter`;

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
        `Overpass API timed out. The query might be too large. Try again later.`,
      );
    }
    throw new Error(`Overpass failed: ${resp.status}`);
  }

  return resp;
}
