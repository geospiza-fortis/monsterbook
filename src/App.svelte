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

  // NOTE: tihs is quite a few canvas calls...
  function mse(canvasA, canvasB) {
    // the mean-squared error
    // assume both canvases are the same size
    let w = canvasA.width;
    let h = canvasA.height;
    let dataA = canvasA.getContext("2d").getImageData(0, 0, w, h).data;
    let dataB = canvasB.getContext("2d").getImageData(0, 0, w, h).data;

    // only need to compare a single channel of the image
    let acc = 0;
    for (let i = 0; i < dataA.length; i += 4) {
      // for consistency with the python code, we deal with numbers betwen 0-255
      acc += Math.pow(dataA[i] - dataB[i], 2);
    }
    return acc / (w * h);
  }

  async function match(test, images) {
    let testCanvas = await canvasFromImage(test);
    let diffs = images.map(async (img) =>
      mse(testCanvas, await canvasFromImage(img))
    );
    let index = diffs.indexOf(Math.min(diffs));
    return index;
  }

  function crop_tag() {}

  async function filter_cards(cards, threshold) {
    let emptyCanvas = rgb2gray(await canvasFromImage(empty));
    let res = [];
    for (let i = 0; i < cards.length; i++) {
      let cardCanvas = rgb2gray(await canvasFromImage(cards[i]));
      let err = mse(cardCanvas, emptyCanvas);
      console.log(err);
      if (err > threshold) {
        res.push(cards[i]);
      }
    }
    return res;
  }

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
      // TODO: what is the correct threshold here? I don't want to pull in a new dependency.
      let filtered = await filter_cards(cards, 100);
      files.push({ img: img, cards: filtered });
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
