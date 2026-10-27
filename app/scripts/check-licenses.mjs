// License gate for npm dependencies. ZaNytky is proprietary software
// (LICENSE.md), so every package shipped in the app bundle must have a
// permissive license. Build-only (dev) packages may also use data licenses.
// Usage: node scripts/check-licenses.mjs   (run in app/)
import { readFileSync } from "node:fs";

const PERMISSIVE = new Set([
  "MIT", "MIT-0", "ISC", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "0BSD",
  "Zlib", "CC0-1.0", "Unlicense", "BlueOak-1.0.0", "Python-2.0",
]);
// Data-only licenses accepted for build tools that are not shipped.
const DEV_ONLY = new Set(["CC-BY-4.0"]);
// Packages whose lockfile entry lacks a license field, verified upstream.
const VERIFIED = { khroma: "MIT" };

/** True if an SPDX expression can be satisfied by the allowed set. */
function satisfies(expr, allowed) {
  const alternatives = expr.replace(/[()]/g, "").split(/\s+OR\s+/);
  return alternatives.some((alt) => alt.split(/\s+AND\s+/).every((id) => allowed.has(id.trim())));
}

const lock = JSON.parse(readFileSync("package-lock.json", "utf8"));
const problems = [];
for (const [key, pkg] of Object.entries(lock.packages)) {
  if (key === "") continue;
  const name = key.slice(key.lastIndexOf("node_modules/") + "node_modules/".length);
  const license = pkg.license ?? VERIFIED[name];
  const allowed = pkg.dev ? new Set([...PERMISSIVE, ...DEV_ONLY]) : PERMISSIVE;
  if (!license || !satisfies(license, allowed)) {
    problems.push(`${name}@${pkg.version}: ${license ?? "no license"}${pkg.dev ? " (dev)" : ""}`);
  }
}
if (problems.length > 0) {
  console.error("Dependencies with licenses not allowed for ZaNytky:\n  " + problems.join("\n  "));
  process.exit(1);
}
console.log(`License check passed for ${Object.keys(lock.packages).length - 1} npm packages.`);
