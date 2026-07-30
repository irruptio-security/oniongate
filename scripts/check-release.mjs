#!/usr/bin/env node

import { readFile } from "node:fs/promises";

const root = new URL("../", import.meta.url);
const packageJson = JSON.parse(
  await readFile(new URL("package.json", root), "utf8"),
);
const packageLock = JSON.parse(
  await readFile(new URL("package-lock.json", root), "utf8"),
);
const tauriConfig = JSON.parse(
  await readFile(new URL("src-tauri/tauri.conf.json", root), "utf8"),
);
const cargoToml = await readFile(new URL("src-tauri/Cargo.toml", root), "utf8");
const changelog = await readFile(new URL("CHANGELOG.md", root), "utf8");

const cargoVersion = cargoToml.match(
  /^\[package\][\s\S]*?^version\s*=\s*"([^"]+)"/m,
)?.[1];
const versions = new Map([
  ["package.json", packageJson.version],
  ["package-lock.json", packageLock.version],
  ["package-lock root package", packageLock.packages?.[""]?.version],
  ["src-tauri/Cargo.toml", cargoVersion],
  ["src-tauri/tauri.conf.json", tauriConfig.version],
]);

const expected = packageJson.version;
const failures = [];
for (const [file, version] of versions) {
  if (version !== expected) {
    failures.push(`${file} has version ${version ?? "(missing)"}, expected ${expected}`);
  }
}

const requested = (process.argv[2] || process.env.RELEASE_TAG || `v${expected}`).trim();
const tag = requested.startsWith("v") ? requested : `v${requested}`;
if (tag !== `v${expected}`) {
  failures.push(`release tag ${tag} does not match application version v${expected}`);
}

const escapedVersion = expected.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
const releaseHeading = new RegExp(
  `^## \\[${escapedVersion}\\] - \\d{4}-\\d{2}-\\d{2}$`,
  "m",
);
if (!releaseHeading.test(changelog)) {
  failures.push(
    `CHANGELOG.md needs an exact "## [${expected}] - YYYY-MM-DD" release section`,
  );
}

if (failures.length > 0) {
  console.error("Release preflight failed:");
  for (const failure of failures) console.error(`- ${failure}`);
  process.exit(1);
}

console.log(`Release preflight passed for ${tag}`);
