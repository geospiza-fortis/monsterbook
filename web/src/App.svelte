<script>
  import { onDestroy } from "svelte";
  import { PAGE_TABS, TOTAL_PAGES } from "./lib/mock-session.js";
  import * as client from "./lib/session-client.js";

  const TAB_COLORS = {
    red: "#d9534f",
    orange: "#e8853c",
    lightgreen: "#8fce5a",
    green: "#3f9142",
    lightblue: "#6cc3e0",
    blue: "#3f6fd0",
    purple: "#9b59c9",
    black: "#7a7a7a",
    gold: "#d4af37",
  };

  let missingSet = new Set(Array.from({ length: TOTAL_PAGES }, (_, i) => i));
  let notices = [];
  let noticeId = 0;
  let rows = [];
  let sortKey = "uid";
  let sortDir = 1;
  let cardsPerRow = 20;
  let includeEmpty = true;
  let stitchedUrl = null;
  let tablePage = 0;
  const PAGE_SIZE = 40;
  let dragging = false;
  let busy = 0;
  let fileInput;

  $: ingestedCount = TOTAL_PAGES - missingSet.size;
  $: sortedRows = [...rows].sort((a, b) => {
    const va = a[sortKey], vb = b[sortKey];
    return (va < vb ? -1 : va > vb ? 1 : 0) * sortDir;
  });
  $: tablePages = Math.max(1, Math.ceil(sortedRows.length / PAGE_SIZE));
  $: tablePage = Math.min(tablePage, tablePages - 1);
  $: pagedRows = sortedRows.slice(
    tablePage * PAGE_SIZE,
    (tablePage + 1) * PAGE_SIZE,
  );

  function notify(message) {
    notices = [...notices, { id: noticeId++, message }];
  }
  function dismiss(id) {
    notices = notices.filter((n) => n.id !== id);
  }

  function setSort(key) {
    if (sortKey === key) sortDir = -sortDir;
    else { sortKey = key; sortDir = 1; }
  }

  async function refresh() {
    missingSet = new Set(await client.missing());
    rows = await client.transcribe();
    await restitch();
  }

  async function restitch() {
    let png;
    try {
      png = await client.stitch(cardsPerRow, includeEmpty);
    } catch (err) {
      // The wasm session has nothing to stitch until a page is ingested.
      if (err?.kind === "NoPageFound") {
        if (stitchedUrl) URL.revokeObjectURL(stitchedUrl);
        stitchedUrl = null;
        return;
      }
      throw err;
    }
    if (stitchedUrl) URL.revokeObjectURL(stitchedUrl);
    stitchedUrl = URL.createObjectURL(new Blob([png], { type: "image/png" }));
  }

  async function ingestBlob(blob, label) {
    busy++;
    try {
      // Decode in the browser (handles webp/avif/anything the browser can
      // render, which the wasm-side image crate cannot), then hand raw RGBA
      // to the session. Fall back to wasm-side decoding if that fails.
      try {
        const bitmap = await createImageBitmap(blob);
        const canvas = new OffscreenCanvas(bitmap.width, bitmap.height);
        const ctx = canvas.getContext("2d");
        ctx.drawImage(bitmap, 0, 0);
        const { data } = ctx.getImageData(0, 0, bitmap.width, bitmap.height);
        bitmap.close();
        await client.addBitmap(canvas.width, canvas.height, data.buffer);
      } catch (decodeErr) {
        if (decodeErr?.kind) throw decodeErr; // session error, not a decode failure
        const buffer = await blob.arrayBuffer();
        await client.addScreenshot(buffer);
      }
    } catch (err) {
      notify(`${label}: ${err?.kind ?? "error"}${err?.message ? ` (${err.message})` : ""}`);
    } finally {
      busy--;
    }
  }

  async function ingestFiles(files) {
    for (const file of files) {
      if (!file.type.startsWith("image/")) {
        notify(`${file.name}: not an image`);
        continue;
      }
      await ingestBlob(file, file.name);
    }
    await refresh();
  }

  function onFileChange(event) {
    ingestFiles([...event.target.files]);
    event.target.value = "";
  }

  function onDrop(event) {
    dragging = false;
    ingestFiles([...event.dataTransfer.files]);
  }

  async function onPaste(event) {
    const blobs = [...event.clipboardData.items]
      .filter((item) => item.type.startsWith("image/"))
      .map((item) => item.getAsFile())
      .filter(Boolean);
    if (!blobs.length) return;
    for (const blob of blobs) await ingestBlob(blob, "pasted image");
    await refresh();
  }

  async function onCardsPerRowChange() {
    if (cardsPerRow >= 1) await restitch();
  }

  refresh();
  onDestroy(() => stitchedUrl && URL.revokeObjectURL(stitchedUrl));
</script>

<svelte:window
  on:paste={onPaste}
  on:dragover|preventDefault={() => (dragging = true)}
  on:dragleave={() => (dragging = false)}
  on:drop|preventDefault={onDrop}
/>

<main class:dragging>
  <h1>Monster Book Stitcher</h1>
  <p>
    This app transcribes in-game screenshots of the Monster Book in
    MapleLegends to help plan and track card hunting. All processing occurs
    on-device and can be run offline.
  </p>
  <p class="muted">
    Add screenshots of your monster book pages: choose files, drag and drop
    them anywhere on the page, or paste from the clipboard.
  </p>

  <div class="ingest">
    <button on:click={() => fileInput.click()}>Choose screenshots…</button>
    <input
      bind:this={fileInput}
      type="file"
      accept="image/*"
      multiple
      hidden
      on:change={onFileChange}
    />
    {#if busy > 0}<span class="muted">processing…</span>{/if}
  </div>

  {#each notices as notice (notice.id)}
    <div class="notice">
      <span>{notice.message}</span>
      <button class="dismiss" on:click={() => dismiss(notice.id)}>×</button>
    </div>
  {/each}

  <h2>Pages ({ingestedCount}/{TOTAL_PAGES})</h2>
  <div class="grid">
    {#each PAGE_TABS as tab, i}
      <div
        class="slot"
        class:missing={missingSet.has(i)}
        style="--tab: {TAB_COLORS[tab]}"
        title="Page {i + 1} ({tab}) — {missingSet.has(i) ? 'missing' : 'ingested'}"
      >
        {i + 1}
      </div>
    {/each}
  </div>

  {#if rows.length}
    <h2>Stitched image</h2>
    <div class="stitch-controls">
      <label>
        Cards per row
        <input
          type="number"
          min="1"
          max="40"
          bind:value={cardsPerRow}
          on:change={onCardsPerRowChange}
        />
      </label>
      <label>
        <input type="checkbox" bind:checked={includeEmpty} on:change={restitch} />
        Include empty cards
      </label>
    </div>
    {#if stitchedUrl}
      <p>
        <a href={stitchedUrl} download="monsterbook.png"><button>Download PNG</button></a>
      </p>
      <img class="stitched" src={stitchedUrl} alt="Stitched monster book" />
    {/if}

    <h2>Cards</h2>
    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            {#each ["uid", "name", "count"] as key}
              <th on:click={() => setSort(key)}>
                {key}{sortKey === key ? (sortDir > 0 ? " ▲" : " ▼") : ""}
              </th>
            {/each}
          </tr>
        </thead>
        <tbody>
          {#each pagedRows as row (row.uid)}
            <tr><td>{row.uid}</td><td>{row.name}</td><td>{row.count}</td></tr>
          {/each}
        </tbody>
      </table>
    </div>
    <div class="pager">
      <button disabled={tablePage === 0} on:click={() => tablePage--}>‹ Prev</button>
      <span class="muted">
        {tablePage + 1} / {tablePages} ({sortedRows.length} cards)
      </span>
      <button disabled={tablePage >= tablePages - 1} on:click={() => tablePage++}>Next ›</button>
    </div>
  {/if}

  <footer class="muted">
    <p>
      The source code can be found on GitHub at
      <a href="https://github.com/geospiza-fortis/monsterbook"
        >geospiza-fortis/monsterbook</a
      >. The monster and map information is taken from the
      <a
        href="https://forum.maplelegends.com/index.php?threads/monster-book-efficient-farming-guide.23984"
        >Monster Book Efficient Farming Guide</a
      >
      by Precel and Bambo (<a
        href="https://docs.google.com/spreadsheets/d/1ohipSCqwiyyOdqNTWrTzDNGUtYJOojfk9qbVHSl70l0/edit#gid=1847158424"
        >link</a
      >).
    </p>
  </footer>
</main>

<style>
  main {
    max-width: 56rem;
    margin: 0 auto;
    padding: 1.5rem 1rem 4rem;
    border: 2px dashed transparent;
    border-radius: 8px;
  }
  main.dragging {
    border-color: var(--accent);
  }
  h1 {
    font-size: 1.4rem;
  }
  h2 {
    font-size: 1.1rem;
    margin-top: 2rem;
  }
  .muted {
    color: var(--muted);
  }
  .ingest {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }
  button {
    font: inherit;
    padding: 0.4rem 0.9rem;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--card);
    color: var(--fg);
    cursor: pointer;
  }
  button:hover {
    border-color: var(--accent);
  }
  .notice {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.5rem;
    margin: 0.5rem 0;
    padding: 0.4rem 0.75rem;
    border: 1px solid #c0605c;
    border-radius: 6px;
    background: color-mix(in srgb, #c0605c 12%, var(--card));
  }
  .dismiss {
    border: none;
    background: none;
    padding: 0 0.25rem;
    font-size: 1.1rem;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(3rem, 1fr));
    gap: 0.4rem;
  }
  .slot {
    aspect-ratio: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 6px;
    background: var(--tab);
    color: #111;
    font-weight: 600;
    border: 2px solid var(--tab);
  }
  .slot.missing {
    background: transparent;
    color: var(--muted);
    opacity: 0.6;
  }
  .table-wrap {
    overflow-x: auto;
  }
  table {
    border-collapse: collapse;
    min-width: 24rem;
  }
  th,
  td {
    text-align: left;
    padding: 0.3rem 0.9rem 0.3rem 0;
    border-bottom: 1px solid var(--border);
  }
  th {
    cursor: pointer;
    user-select: none;
  }
  input[type="number"] {
    font: inherit;
    width: 4.5rem;
    padding: 0.25rem 0.4rem;
    background: var(--card);
    color: var(--fg);
    border: 1px solid var(--border);
    border-radius: 6px;
  }
  .pager {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-top: 0.75rem;
  }
  .pager button:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .stitch-controls {
    display: flex;
    align-items: center;
    gap: 1.5rem;
    flex-wrap: wrap;
    margin-bottom: 0.75rem;
  }
  footer {
    margin-top: 3rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border);
    font-size: 0.9rem;
  }
  footer a {
    color: var(--accent);
  }
  .stitched {
    max-width: 100%;
    border: 1px solid var(--border);
    border-radius: 6px;
  }
</style>
