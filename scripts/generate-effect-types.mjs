#!/usr/bin/env node
/**
 * Runs `cargo test export_bindings` with TS_RS_EXPORT_DIR pointed at
 * src/domain/effects, then appends re-exports of those types into types.ts
 * so the rest of the project can import everything from one place.
 *
 * Called automatically by `npm run generate-types` via the
 * `generate-effect-types` script in package.json.
 */

import {execSync} from "node:child_process";
import {mkdirSync, readdirSync, readFileSync, writeFileSync} from "node:fs";
import {basename, dirname, extname, resolve} from "node:path";
import {fileURLToPath} from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(__dirname, "..");
const effectsDir = resolve(projectRoot, "src", "domain", "effects");
const typesFile = resolve(projectRoot, "src", "domain", "types.ts");

// 1. Ensure the output directory exists
mkdirSync(effectsDir, { recursive: true });

// 2. Run ts-rs export tests
console.log(`[effect-types] Exporting ts-rs bindings to ${effectsDir}`);
execSync("cargo test export_bindings --quiet --no-default-features", {
  cwd: resolve(projectRoot, "src-tauri"),
  env: { ...process.env, TS_RS_EXPORT_DIR: effectsDir },
  stdio: "inherit",
});

// 3. Collect every .ts file that was generated in the effects directory.
// Skip EffectDto.ts — tauri-typegen already emits the EffectDto union in types.ts
// and a second export would cause a TypeScript duplicate-identifier error.
const SKIP = new Set(["EffectDto.ts", "IrProfileDto.ts"]);
const generatedFiles = readdirSync(effectsDir)
  .filter((f) => extname(f) === ".ts" && !SKIP.has(f))
  .sort();

if (generatedFiles.length === 0) {
  console.warn("[effect-types] No .ts files found in effects dir — nothing to append.");
  process.exit(0);
}

// 4. Build import + re-export lines.
//    The import makes the type available to the EffectDto union already in types.ts.
//    The re-export makes it available to every other file that imports from types.ts.
const imports = generatedFiles
  .map((f) => {
    const typeName = basename(f, ".ts");
    return `import type { ${typeName} } from './effects/${typeName}';`;
  })
  .join("\n");

const reExports = generatedFiles
  .map((f) => {
    const typeName = basename(f, ".ts");
    return `export type { ${typeName} };`;
  })
  .join("\n");

const marker = "// [effect-types] -- auto-appended by scripts/generate-effect-types.mjs";
let types = readFileSync(typesFile, "utf8");

// Strip any previous prepend so we don't accumulate duplicates on re-runs
const markerIndex = types.indexOf(marker);
if (markerIndex !== -1) {
  types = types.slice(markerIndex + marker.length).trimStart();
}

writeFileSync(typesFile, `${marker}\n${imports}\n${reExports}\n\n${types}`);
console.log(`[effect-types] Appended ${generatedFiles.length} effect type export(s) to types.ts`);
console.log(generatedFiles.map((f) => `  - ${f}`).join("\n"));
