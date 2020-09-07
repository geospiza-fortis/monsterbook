<script>
  import { reference, empty, seed_tags, entries } from "./assets.js";
  import { metadata, offsets } from "./metadata.js";
  import {
    readDataAsync,
    readImageAsync,
    canvasFromImage,
    cloneCanvas,
    sobel,
    rgb2gray,
    cropImage,
  } from "./image.js";
  import { onMount } from "svelte";
  import { claim_text } from "svelte/internal";

  let canvas;

  async function crop(dataUrl) {
    // TODO: adapt this for all resolutions and non-macs
    return await cropImage(dataUrl, 412, 862, 569, 899);
  }

  async function crop_cards(dataUrl) {
    const num_rows = 5;
    const num_cols = 5;
    let img = await readImageAsync(dataUrl);
    const h = img.height / num_rows;
    const w = img.width / num_cols;
    let cards = [];
    for (let i = 0; i < num_rows; i++) {
      for (let j = 0; j < num_cols; j++) {
        let card = await cropImage(
          dataUrl,
          i * h,
          (i + 1) * h,
          j * w,
          (j + 1) * w
        );
        cards.push(card);
      }
    }
    return cards;
  }

  function match(test, images) {}

  function crop_tag() {}

  function filter_cards() {}

  function transcribe() {}

  let files = [];
  let progress = 0;
  let total = 0;
  let error = 0;

  async function appendToFiles(filelist) {
    error = 0;
    total = filelist.length;
    for (let i = 0; i < filelist.length; i++) {
      progress = i;
      let file = filelist[i];

      let dataUrl = await readDataAsync(file);
      let uncropped = await readImageAsync(dataUrl);
      let img = await crop(dataUrl);
      let cards = await crop_cards(img);
      files.push({ img: img, cards: cards });
    }
    files = files;
    progress = total;
  }

  async function handleTranscribe(event) {
    event.preventDefault();
    let fileInput = document.getElementById("fileInput");
    await appendToFiles(fileInput.files);
  }

  onMount(async () => {
    canvas = await canvasFromImage(reference[0]);
    sobel(rgb2gray(canvas));
  });
</script>

<h1>Monsterbook Transcription</h1>

<form on:submit={handleTranscribe}>
  <input id="fileInput" type="file" accept="image/png" name="file[]" multiple />
  <input type="submit" value="Transcribe" />
</form>

{#if total > 0}
  <p>
    {#if progress < total}
      Processing {progress}/{total}
      {#if error}with {error} errors{/if} ...
    {:else}Processed {progress}/{total} with {error} errors.{/if}
  </p>
{/if}

{#each files as file}
  {#each file.cards as card}<img src={card} />{/each}
{/each}

<h2>Debugging stuff</h2>
{#if canvas}<img src={canvas.toDataURL()} />{/if}
<img src={reference[0]} />
<img src={empty} />
<img src={seed_tags[0]} />
<pre>{entries}</pre>
