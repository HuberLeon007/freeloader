// SPDX-License-Identifier: GPL-3.0-or-later
// Generates the Freeloader application icon set.
//
// Deliberately dependency free: it rasterises the mark with 4x supersampling
// and encodes PNG/ICO using only node:zlib. Tauri bundling needs real icons on
// every platform, and shipping binaries through review is worse than deriving
// them from 120 lines of maths.

import { deflateSync } from "node:zlib";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const outDir = join(here, "..", "apps", "desktop", "src-tauri", "icons");

const BACKDROP_TOP = [0x14, 0xe0, 0x9a];
const BACKDROP_BOTTOM = [0x0f, 0xa7, 0xc4];
const GLYPH = [0x06, 0x0a, 0x0d];

const clamp01 = (value) => (value < 0 ? 0 : value > 1 ? 1 : value);

function roundedRectCoverage(x, y, radius) {
  const inset = 0.06;
  const min = inset;
  const max = 1 - inset;
  if (x < min || x > max || y < min || y > max) return false;
  const cx = Math.min(Math.max(x, min + radius), max - radius);
  const cy = Math.min(Math.max(y, min + radius), max - radius);
  const dx = x - cx;
  const dy = y - cy;
  return dx * dx + dy * dy <= radius * radius;
}

function inGlyph(x, y) {
  // Stem of the download arrow.
  if (x >= 0.445 && x <= 0.555 && y >= 0.215 && y <= 0.545) return true;
  // Arrow head: isosceles triangle pointing down.
  if (y >= 0.5 && y <= 0.735) {
    const t = (y - 0.5) / 0.235;
    const half = 0.19 * (1 - t);
    if (Math.abs(x - 0.5) <= half) return true;
  }
  // Baseline tray.
  if (x >= 0.285 && x <= 0.715 && y >= 0.785 && y <= 0.855) return true;
  return false;
}

function renderRgba(size) {
  const samples = 4;
  const pixels = Buffer.alloc(size * size * 4);
  const radius = 0.2;
  for (let py = 0; py < size; py += 1) {
    for (let px = 0; px < size; px += 1) {
      let alpha = 0;
      let glyph = 0;
      for (let sy = 0; sy < samples; sy += 1) {
        for (let sx = 0; sx < samples; sx += 1) {
          const x = (px + (sx + 0.5) / samples) / size;
          const y = (py + (sy + 0.5) / samples) / size;
          if (!roundedRectCoverage(x, y, radius)) continue;
          alpha += 1;
          if (inGlyph(x, y)) glyph += 1;
        }
      }
      const total = samples * samples;
      const coverage = alpha / total;
      const offset = (py * size + px) * 4;
      if (coverage === 0) continue;
      const gradient = clamp01(py / (size - 1));
      const glyphMix = glyph / total / (coverage || 1);
      for (let channel = 0; channel < 3; channel += 1) {
        const base =
          BACKDROP_TOP[channel] * (1 - gradient) + BACKDROP_BOTTOM[channel] * gradient;
        const mixed = base * (1 - glyphMix) + GLYPH[channel] * glyphMix;
        pixels[offset + channel] = Math.round(clamp01(mixed / 255) * 255);
      }
      pixels[offset + 3] = Math.round(coverage * 255);
    }
  }
  return pixels;
}

const CRC_TABLE = (() => {
  const table = new Int32Array(256);
  for (let n = 0; n < 256; n += 1) {
    let c = n;
    for (let k = 0; k < 8; k += 1) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    table[n] = c;
  }
  return table;
})();

function crc32(buffer) {
  let crc = -1;
  for (let i = 0; i < buffer.length; i += 1) {
    crc = CRC_TABLE[(crc ^ buffer[i]) & 0xff] ^ (crc >>> 8);
  }
  return (crc ^ -1) >>> 0;
}

function chunk(type, data) {
  const length = Buffer.alloc(4);
  length.writeUInt32BE(data.length, 0);
  const body = Buffer.concat([Buffer.from(type, "ascii"), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(body), 0);
  return Buffer.concat([length, body, crc]);
}

function encodePng(size, rgba) {
  const raw = Buffer.alloc((size * 4 + 1) * size);
  for (let y = 0; y < size; y += 1) {
    raw[y * (size * 4 + 1)] = 0;
    rgba.copy(raw, y * (size * 4 + 1) + 1, y * size * 4, (y + 1) * size * 4);
  }
  const header = Buffer.alloc(13);
  header.writeUInt32BE(size, 0);
  header.writeUInt32BE(size, 4);
  header[8] = 8;
  header[9] = 6;
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk("IHDR", header),
    chunk("IDAT", deflateSync(raw, { level: 9 })),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

function encodeIco(entries) {
  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0);
  header.writeUInt16LE(1, 2);
  header.writeUInt16LE(entries.length, 4);
  const directory = Buffer.alloc(16 * entries.length);
  let offset = header.length + directory.length;
  entries.forEach((entry, index) => {
    const at = index * 16;
    directory[at] = entry.size >= 256 ? 0 : entry.size;
    directory[at + 1] = entry.size >= 256 ? 0 : entry.size;
    directory.writeUInt16LE(1, at + 4);
    directory.writeUInt16LE(32, at + 6);
    directory.writeUInt32BE(0, at + 8);
    directory.writeUInt32LE(entry.png.length, at + 8);
    directory.writeUInt32LE(offset, at + 12);
    offset += entry.png.length;
  });
  return Buffer.concat([header, directory, ...entries.map((entry) => entry.png)]);
}

const cache = new Map();
function png(size) {
  const existing = cache.get(size);
  if (existing) return existing;
  const created = encodePng(size, renderRgba(size));
  cache.set(size, created);
  return created;
}

mkdirSync(outDir, { recursive: true });

const pngTargets = [
  ["32x32.png", 32],
  ["128x128.png", 128],
  ["128x128@2x.png", 256],
  ["icon.png", 512],
];

for (const [name, size] of pngTargets) {
  writeFileSync(join(outDir, name), png(size));
}

writeFileSync(
  join(outDir, "icon.ico"),
  encodeIco([16, 32, 48, 64, 128, 256].map((size) => ({ size, png: png(size) }))),
);

console.log(`Freeloader icons written to ${outDir}`);
