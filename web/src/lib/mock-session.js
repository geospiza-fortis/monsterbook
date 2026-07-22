// Mock implementation of the future wasm-pack `Session` module.
// The real module (built with wasm-pack in a later phase) exposes the same
// interface; swap the import in worker.js to replace this mock.

export const TOTAL_PAGES = 26;

// Tab colors for the 26 monster book pages, in order. Matches
// monsterbook/src/assets/book.json (Phase 5: tab colors inferred from the
// ribbon signal, validated 15/15 against confirmed old->new page mappings;
// see data/processed/reference_new/REPORT.md).
export const PAGE_TABS = [
  "red",
  "orange", "orange", "orange",
  "lightgreen", "lightgreen", "lightgreen", "lightgreen", "lightgreen",
  "green", "green", "green", "green",
  "lightblue", "lightblue", "lightblue",
  "blue", "blue",
  "purple", "purple",
  "black", "black",
  "gold", "gold", "gold", "gold",
].slice(0, TOTAL_PAGES);

const MOB_NAMES = [
  "Snail", "Blue Snail", "Red Snail", "Shroom", "Orange Mushroom",
  "Slime", "Stump", "Pig", "Ribbon Pig", "Octopus",
  "Green Mushroom", "Horny Mushroom", "Zombie Mushroom", "Axe Stump",
  "Wild Boar", "Fire Boar", "Evil Eye", "Curse Eye", "Jr. Necki",
  "Ligator", "Croco", "Lupin", "Zombie Lupin", "Wraith",
  "Skeleton Soldier", "Mummydog", "Sand Rat", "Drake", "Copper Drake",
  "Red Drake", "Wild Kargo", "Tauromacis", "Taurospear", "Crimson Balrog",
  "Jr. Yeti", "Yeti", "Werewolf", "Lycanthrope", "Cerebes", "Bain",
];

export class Session {
  constructor() {
    this._ingested = new Set();
  }

  _nextMissing() {
    for (let i = 0; i < TOTAL_PAGES; i++) {
      if (!this._ingested.has(i)) return i;
    }
    return null;
  }

  _ingest() {
    const pageId = this._nextMissing();
    if (pageId === null) {
      throw { kind: "NoPageFound" };
    }
    this._ingested.add(pageId);
    return { page_id: pageId };
  }

  add_screenshot(bytes) {
    if (!(bytes instanceof Uint8Array) || bytes.length === 0) {
      throw { kind: "DecodeError" };
    }
    return this._ingest();
  }

  add_bitmap(width, height, rgba) {
    if (!width || !height || rgba.length !== width * height * 4) {
      throw { kind: "DecodeError" };
    }
    return this._ingest();
  }

  missing() {
    const out = [];
    for (let i = 0; i < TOTAL_PAGES; i++) {
      if (!this._ingested.has(i)) out.push(i);
    }
    return out;
  }

  transcribe() {
    // Plausible fake data: a few entries per ingested page.
    const rows = [];
    for (const pageId of [...this._ingested].sort((a, b) => a - b)) {
      for (let slot = 0; slot < 5; slot++) {
        const uid = pageId * 5 + slot;
        rows.push({
          uid,
          name: MOB_NAMES[uid % MOB_NAMES.length],
          count: ((uid * 7919) % 6),
        });
      }
    }
    return rows;
  }

  stitch(cardsPerRow) {
    // The real wasm module returns PNG bytes synchronously; the mock draws a
    // placeholder grid on an OffscreenCanvas. This is async, so callers should
    // `await` the result (harmless once the real module is sync).
    const n = this._ingested.size * 5;
    const cols = Math.max(1, cardsPerRow | 0);
    const rows = Math.max(1, Math.ceil(Math.max(n, 1) / cols));
    const cw = 60, ch = 84, pad = 6;
    const canvas = new OffscreenCanvas(cols * (cw + pad) + pad, rows * (ch + pad) + pad);
    const ctx = canvas.getContext("2d");
    ctx.fillStyle = "#1e2229";
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    for (let i = 0; i < Math.max(n, 1); i++) {
      const x = pad + (i % cols) * (cw + pad);
      const y = pad + Math.floor(i / cols) * (ch + pad);
      ctx.fillStyle = `hsl(${(i * 37) % 360} 45% 55%)`;
      ctx.fillRect(x, y, cw, ch);
      ctx.fillStyle = "#111";
      ctx.font = "12px sans-serif";
      ctx.fillText(String(i), x + 4, y + 14);
    }
    return canvas
      .convertToBlob({ type: "image/png" })
      .then((blob) => blob.arrayBuffer())
      .then((buf) => new Uint8Array(buf));
  }
}
