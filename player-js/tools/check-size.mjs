// Fails when the built loader, gzipped at level 9, exceeds its budget in docs/export.md.
import { readFileSync } from 'node:fs';
import { gzipSync } from 'node:zlib';

const LOADER = new URL('../life-pixel.js', import.meta.url);
const BUDGET_BYTES = 2048;
const GZIP_LEVEL = 9;

const size = gzipSync(readFileSync(LOADER), { level: GZIP_LEVEL }).length;
const report = `life-pixel.js: ${size} bytes gzipped, budget ${BUDGET_BYTES}`;
if (size > BUDGET_BYTES) {
  console.error(`${report}: over budget by ${size - BUDGET_BYTES} bytes`);
  process.exit(1);
}
console.log(report);
