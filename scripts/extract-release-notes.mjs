#!/usr/bin/env node

import { readFile, writeFile } from "node:fs/promises";

const version = (process.argv[2] || "").replace(/^v/, "");
const output = process.argv[3] || "release-notes.md";
if (!version) {
  console.error("Usage: node scripts/extract-release-notes.mjs <version> [output]");
  process.exit(2);
}

const changelog = await readFile(
  new URL("../CHANGELOG.md", import.meta.url),
  "utf8",
);
const lines = changelog.split(/\r?\n/);
const heading = new RegExp(
  `^## \\[${version.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}\\] - \\d{4}-\\d{2}-\\d{2}$`,
);
const start = lines.findIndex((line) => heading.test(line));
const end =
  start < 0
    ? -1
    : lines.findIndex((line, index) => index > start && line.startsWith("## ["));
const notes =
  start < 0 ? "" : lines.slice(start + 1, end < 0 ? undefined : end).join("\n").trim();

if (!notes) {
  console.error(`No non-empty changelog section found for ${version}`);
  process.exit(1);
}

await writeFile(output, `${notes}\n`);
console.log(`Wrote ${output} from CHANGELOG.md ${version}`);
