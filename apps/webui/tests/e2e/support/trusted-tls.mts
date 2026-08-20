import {
  createHash,
  generateKeyPairSync,
  randomBytes,
  X509Certificate,
} from "node:crypto";
import dns from "node:dns";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createServer, request as httpRequest } from "node:http";
import { connect } from "node:net";
import { join, resolve } from "node:path";
import { getCACertificates, setDefaultCACertificates } from "node:tls";
import forge from "node-forge";

/**
 * One TLS path for every platform and for CI: a short-lived CA under the
 * ignored temp directory that only the test processes are told to trust. No
 * operating-system trust store, hosts file or privileged command is involved.
 */
const names = ["ui.konobangu.test", "api.konobangu-api.test"] as const;
// The identity provider is reached by its loopback name, never remapped.
const issuerName = "localhost";
const directory = resolve(
  import.meta.dirname,
  "../../../../../temp/verification/browser/tls",
);

export interface TrustedTlsFixture extends AsyncDisposable {
  readonly cert: string;
  readonly key: string;
  /** The CA the API server is configured to trust for the issuer. */
  readonly caPath: string;
  /** A forward proxy for engines that cannot map the fixture names themselves. */
  readonly nameProxy: string;
}

function rsaKeyPair() {
  // Native key generation; the library only assembles and signs certificates.
  const pair = generateKeyPairSync("rsa", {
    modulusLength: 2048,
    publicKeyEncoding: { type: "spki", format: "pem" },
    privateKeyEncoding: { type: "pkcs8", format: "pem" },
  });
  return {
    privateKeyPem: pair.privateKey,
    publicKey: forge.pki.publicKeyFromPem(pair.publicKey),
    privateKey: forge.pki.privateKeyFromPem(pair.privateKey),
  };
}

function certificate(
  commonName: string,
  publicKey: forge.pki.PublicKey,
  days: number,
) {
  const result = forge.pki.createCertificate();
  result.publicKey = publicKey;
  // A positive serial: the leading hex digit keeps the sign bit clear.
  result.serialNumber = `01${randomBytes(15).toString("hex")}`;
  result.validity.notBefore = new Date(Date.now() - 60_000);
  result.validity.notAfter = new Date(Date.now() + days * 24 * 60 * 60 * 1000);
  result.setSubject([{ name: "commonName", value: commonName }]);
  return result;
}

/**
 * A short-lived CA and one leaf for the fixture names. The CA key signs that
 * leaf and is then discarded, so nothing else can ever be issued under it.
 * Synchronous so the Playwright configuration can create it before a browser
 * launches.
 */
function generateTlsMaterial(directory: string) {
  const ca = join(directory, "ca.pem");
  const key = join(directory, "leaf.key");
  const cert = join(directory, "leaf.pem");
  const authority = rsaKeyPair();
  const authorityCertificate = certificate(
    "Konobangu browser fixture",
    authority.publicKey,
    2,
  );
  authorityCertificate.setIssuer(authorityCertificate.subject.attributes);
  authorityCertificate.setExtensions([
    {
      name: "basicConstraints",
      cA: true,
      pathLenConstraint: 0,
      critical: true,
    },
    { name: "keyUsage", keyCertSign: true, cRLSign: true, critical: true },
    { name: "subjectKeyIdentifier" },
  ]);
  authorityCertificate.sign(authority.privateKey, forge.md.sha256.create());
  const leaf = rsaKeyPair();
  const leafCertificate = certificate(names[0], leaf.publicKey, 1);
  leafCertificate.setIssuer(authorityCertificate.subject.attributes);
  leafCertificate.setExtensions([
    { name: "basicConstraints", cA: false, critical: true },
    {
      name: "keyUsage",
      digitalSignature: true,
      keyEncipherment: true,
      critical: true,
    },
    { name: "extKeyUsage", serverAuth: true },
    {
      name: "subjectAltName",
      altNames: [...names, issuerName].map((value) => ({ type: 2, value })),
    },
    {
      name: "authorityKeyIdentifier",
      keyIdentifier: authorityCertificate
        .generateSubjectKeyIdentifier()
        .getBytes(),
    },
  ]);
  leafCertificate.sign(authority.privateKey, forge.md.sha256.create());
  writeFileSync(ca, forge.pki.certificateToPem(authorityCertificate));
  writeFileSync(key, leaf.privateKeyPem, { mode: 0o600 });
  writeFileSync(cert, forge.pki.certificateToPem(leafCertificate));
  return { ca, key, cert };
}

/**
 * The Playwright configuration and each worker are separate processes that
 * must agree on one key, so unexpired material on disk is reused.
 */
function tlsMaterial() {
  mkdirSync(directory, { recursive: true });
  const cert = join(directory, "leaf.pem");
  const reusable =
    existsSync(cert) &&
    new X509Certificate(readFileSync(cert)).validToDate.getTime() >
      Date.now() + 60 * 60 * 1000;
  return reusable
    ? { ca: join(directory, "ca.pem"), key: join(directory, "leaf.key"), cert }
    : generateTlsMaterial(directory);
}

/**
 * Chromium maps the fixture names and trusts only the fixture leaf key, both
 * by launch argument.
 */
export function chromiumFixtureArgs(): string[] {
  const spki = createHash("sha256")
    .update(
      new X509Certificate(readFileSync(tlsMaterial().cert)).publicKey.export({
        type: "spki",
        format: "der",
      }),
    )
    .digest("base64");
  return [
    `--host-resolver-rules=${names.map((name) => `MAP ${name} 127.0.0.1`).join(", ")}`,
    `--ignore-certificate-errors-spki-list=${spki}`,
  ];
}

/**
 * Requests issued by this Node process (Playwright's request client) need the
 * fixture names too; they are answered in-process.
 */
function resolveFixtureNames(): Disposable {
  const owned = (hostname: unknown) =>
    typeof hostname === "string" &&
    (names as readonly string[]).includes(hostname);
  const callbackLookup = dns.lookup;
  const promiseLookup = dns.promises.lookup;
  const answer = (options: unknown) => {
    const address = { address: "127.0.0.1", family: 4 };
    return typeof options === "object" &&
      options !== null &&
      "all" in options &&
      options.all
      ? [address]
      : address;
  };
  dns.lookup = ((hostname: string, ...rest: unknown[]) => {
    if (!owned(hostname))
      return Reflect.apply(callbackLookup, dns, [hostname, ...rest]);
    const callback = rest.at(-1) as (...result: unknown[]) => void;
    const result = answer(rest.length > 1 ? rest[0] : undefined);
    process.nextTick(() =>
      Array.isArray(result)
        ? callback(null, result)
        : callback(null, result.address, result.family),
    );
  }) as typeof dns.lookup;
  dns.promises.lookup = (async (hostname: string, options?: unknown) =>
    owned(hostname)
      ? answer(options)
      : Reflect.apply(promiseLookup, dns.promises, [
          hostname,
          options,
        ])) as typeof dns.promises.lookup;
  return {
    [Symbol.dispose]() {
      dns.lookup = callbackLookup;
      dns.promises.lookup = promiseLookup;
    },
  };
}

/**
 * Firefox and WebKit have no launch argument for host mapping. Playwright's
 * proxy option reaches every engine, so a loopback forward proxy answers the
 * fixture names with 127.0.0.1 and passes any other host through unchanged.
 */
async function startNameProxy(
  ownership: AsyncDisposableStack,
): Promise<string> {
  const target = (hostname: string) =>
    (names as readonly string[]).includes(hostname) ? "127.0.0.1" : hostname;
  const server = createServer((request, response) => {
    const url = new URL(request.url ?? "/");
    const upstream = httpRequest(
      {
        host: target(url.hostname),
        port: url.port || 80,
        path: `${url.pathname}${url.search}`,
        method: request.method,
        headers: request.headers,
      },
      (reply) => {
        response.writeHead(reply.statusCode ?? 502, reply.headers);
        reply.pipe(response);
      },
    );
    upstream.once("error", () => response.destroy());
    request.pipe(upstream);
  });
  // Tunnels outlive their HTTP exchange, so the server no longer tracks them.
  const tunnels = new Set<{ destroy(): void }>();
  server.on("connect", (request, client, head) => {
    const authority = new URL(`http://${request.url}`);
    const upstream = connect(
      Number(authority.port || 443),
      target(authority.hostname),
      () => {
        client.write("HTTP/1.1 200 Connection Established\r\n\r\n");
        upstream.write(head);
        upstream.pipe(client);
        client.pipe(upstream);
      },
    );
    for (const socket of [client, upstream]) {
      tunnels.add(socket);
      socket.once("close", () => tunnels.delete(socket));
    }
    upstream.once("error", () => client.destroy());
    client.once("error", () => upstream.destroy());
    upstream.once("close", () => client.destroy());
    client.once("close", () => upstream.destroy());
  });
  await new Promise<void>((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  ownership.defer(
    () =>
      new Promise<void>((resolve) => {
        for (const socket of tunnels) socket.destroy();
        server.closeAllConnections();
        server.close(() => resolve());
      }),
  );
  const address = server.address();
  if (address === null || typeof address === "string")
    throw new Error("The fixture name proxy did not bind a TCP listener");
  return `http://127.0.0.1:${address.port}`;
}

/** Trust is limited to this fixture's CA, names and test lifetime. */
export async function createTrustedTlsFixture(): Promise<TrustedTlsFixture> {
  await using ownership = new AsyncDisposableStack();
  const material = tlsMaterial();
  const cert = readFileSync(material.cert, "utf8");
  const certificate = new X509Certificate(cert);
  for (const name of [...names, issuerName])
    if (!certificate.checkHost(name))
      throw new Error(`The fixture TLS certificate does not cover ${name}`);
  const previous = getCACertificates("default");
  setDefaultCACertificates([...previous, readFileSync(material.ca, "utf8")]);
  ownership.defer(() => setDefaultCACertificates(previous));
  ownership.use(resolveFixtureNames());
  const nameProxy = await startNameProxy(ownership);
  const owner = ownership.move();
  return {
    cert,
    key: readFileSync(material.key, "utf8"),
    caPath: material.ca,
    nameProxy,
    [Symbol.asyncDispose]: () => owner.disposeAsync(),
  };
}
