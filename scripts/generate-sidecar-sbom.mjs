#!/usr/bin/env node

import { randomUUID } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";

const root = new URL("../", import.meta.url);
const manifest = await readFile(
  new URL("scripts/dependencies.sha256", root),
  "utf8",
);
const packageJson = JSON.parse(
  await readFile(new URL("package.json", root), "utf8"),
);

function describe(filename) {
  const tor = filename.match(
    /^tor-expert-bundle-(.+)-(\d+\.\d+\.\d+)\.tar\.gz$/,
  );
  if (tor) {
    return {
      name: "Tor Expert Bundle",
      version: tor[2],
      target: tor[1],
      distribution:
        `https://dist.torproject.org/torbrowser/${tor[2]}/${filename}`,
    };
  }

  const singBox = filename.match(
    /^sing-box-(\d+\.\d+\.\d+)-(.+)\.(?:tar\.gz|zip)$/,
  );
  if (singBox) {
    return {
      name: "sing-box",
      version: singBox[1],
      target: singBox[2],
      distribution:
        `https://github.com/SagerNet/sing-box/releases/download/v${singBox[1]}/${filename}`,
    };
  }

  return {
    name: filename,
    version: "unknown",
    target: "unknown",
    distribution: null,
  };
}

const components = manifest
  .split(/\r?\n/)
  .map((line) => line.trim())
  .filter((line) => line && !line.startsWith("#"))
  .map((line) => {
    const [hash, filename] = line.split(/\s+/, 2);
    if (!/^[a-f0-9]{64}$/.test(hash) || !filename) {
      throw new Error(`Malformed dependency hash line: ${line}`);
    }
    const info = describe(filename);
    return {
      type: "file",
      name: info.name,
      version: info.version,
      "bom-ref": `sidecar:${filename}`,
      hashes: [{ alg: "SHA-256", content: hash }],
      properties: [
        { name: "oniongate:archive", value: filename },
        { name: "oniongate:target", value: info.target },
      ],
      externalReferences: info.distribution
        ? [{ type: "distribution", url: info.distribution }]
        : [],
    };
  });

const bom = {
  bomFormat: "CycloneDX",
  specVersion: "1.6",
  serialNumber: `urn:uuid:${randomUUID()}`,
  version: 1,
  metadata: {
    timestamp: new Date().toISOString(),
    component: {
      type: "application",
      name: "OnionGate",
      version: packageJson.version,
      licenses: [{ license: { id: "GPL-3.0-only" } }],
    },
  },
  components,
};

const output = process.argv[2] || "sidecars.cdx.json";
await writeFile(output, `${JSON.stringify(bom, null, 2)}\n`);
console.log(`Wrote ${output} with ${components.length} pinned archives`);
