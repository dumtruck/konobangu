import { createServer } from "node:http";

/** The production collector consumes these endpoints; no task result is seeded. */
export async function startMikanSource(poster: Buffer) {
  const requests: string[] = [];
  let baseUrl: string;
  await using ownership = new AsyncDisposableStack();
  const server = ownership.use(
    createServer((request, response) => {
      const url = new URL(request.url ?? "/", "http://127.0.0.1");
      requests.push(`${url.pathname}${url.search}`);
      const provider = url.searchParams.get("token");
      if (
        url.pathname === "/RSS/MyBangumi" &&
        (provider === "bundled" || provider === "oidc")
      ) {
        const episode = `runtime-image-${provider}`;
        response.setHeader("Content-Type", "application/rss+xml");
        response.end(`<?xml version="1.0"?><rss version="2.0"><channel>
          <item><title>[Fixture] Runtime image cover - 01 [1080p]</title>
            <link>${baseUrl}Home/Episode/${episode}</link>
            <enclosure type="application/x-bittorrent" length="128" url="${baseUrl}Download/${episode}.torrent"/>
            <torrent><pubDate>2026-10-01T00:00:00Z</pubDate><contentLength>128</contentLength><link>${baseUrl}Download/${episode}.torrent</link></torrent>
          </item></channel></rss>`);
        return;
      }
      if (
        /^\/Home\/Episode\/runtime-image-(bundled|oidc)$/.test(url.pathname)
      ) {
        const episode = url.pathname.split("/").at(-1);
        response.setHeader("Content-Type", "text/html; charset=utf-8");
        response.end(`<!doctype html><title>[Fixture] Runtime image cover - 01 [1080p] - Mikan Project</title>
          <div class="bangumi-title"><a href="/Home/Bangumi/${episode}">Runtime image cover</a>
            <a class="mikan-rss" data-original-title="RSS" href="/RSS/Bangumi?bangumiId=${episode}&amp;subgroupid=202">RSS</a></div>
          <div class="bangumi-poster" data-src="/images/Bangumi/${episode}.png"></div>`);
        return;
      }
      if (
        /^\/images\/Bangumi\/runtime-image-(bundled|oidc)\.png$/.test(
          url.pathname,
        )
      ) {
        response.setHeader("Content-Type", "image/png");
        response.end(poster);
        return;
      }
      response.writeHead(404).end("Unexpected production collector request");
    }),
  );
  await new Promise<void>((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const address = server.address();
  if (address === null || typeof address === "string")
    throw new Error("Mikan fixture did not bind a TCP listener");
  baseUrl = `http://host.testcontainers.internal:${address.port}/`;
  const lifetime = ownership.move();
  return {
    baseUrl,
    port: address.port,
    requests,
    [Symbol.asyncDispose]: () => lifetime.disposeAsync(),
  };
}
