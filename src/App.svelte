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

  async function crop(dataUrl) {
    // TODO: adapt this for all resolutions and non-macs
    return await cropImage(dataUrl, 412, 862, 569, 899);
  }

  async function crop_tag(dataUrl) {
    return await cropImage(dataUrl, 62, 80, 11, 23);
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
    return parseInt(acc / (w * h));
  }

  async function match(test, images, imageFilter = (x) => x) {
    let testCanvas = imageFilter(await canvasFromImage(test));
    let diffs = [];
    for (let i = 0; i < images.length; i++) {
      let refCanvas = imageFilter(await canvasFromImage(images[i]));
      let diff = mse(testCanvas, refCanvas);
      diffs.push(diff);
    }
    let index = diffs.indexOf(Math.min.apply(null, diffs));
    // also return the diffs?
    // console.log(diffs);
    return index;
  }

  async function filter_cards(cards, threshold) {
    let emptyCanvas = rgb2gray(await canvasFromImage(empty));
    let res = [];
    for (let i = 0; i < cards.length; i++) {
      let cardCanvas = rgb2gray(await canvasFromImage(cards[i]));
      let err = mse(cardCanvas, emptyCanvas);
      // console.log(err);
      if (err > threshold) {
        res.push(cards[i]);
      }
    }
    return res;
  }

  async function transcribe(dataUrl) {
    const empty_threshold = 100;
    const unseen_threshold = 5000;

    let uncropped = await readImageAsync(dataUrl);
    let img = await crop(dataUrl);

    // match this against one of the existing files
    let index = await match(img, reference, (canvas) =>
      sobel(rgb2gray(canvas))
    );

    let cards = await crop_cards(img);
    // TODO: what is the correct threshold here? I don't want to pull in a new dependency
    // to rigorously determine the threshold.
    let filtered = await filter_cards(cards, empty_threshold);

    // determine the appropriate tag for each one
    let cardData = [];
    let emptyCanvas = await canvasFromImage(empty);
    for (let i = 0; i < filtered.length; i++) {
      let tag = await crop_tag(filtered[i]);
      let value = 0;
      let err = mse(await canvasFromImage(tag), emptyCanvas, rgb2gray);
      // console.log(err);
      if (err > unseen_threshold) {
        value = (await match(tag, seed_tags)) + 1;
      }
      let data = {
        uid: offsets[index] + i,
        name: entries[offsets[index] + i],
        count: value,
      };
      cardData.push(data);
    }

    return {
      img: img,
      cards: filtered,
      data: cardData,
      metadata: metadata[index],
    };
  }

  let displayTable = true;
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
      let transcribed = await transcribe(dataUrl);
      files.push(transcribed);
    }
    files = files;
    progress = total;
  }

  async function handleTranscribe(event) {
    event.preventDefault();
    let fileInput = document.getElementById("fileInput");
    await appendToFiles(fileInput.files);
  }

  onMount(async () => {});
</script>

<style>
  .wrapper {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
  }
  table {
    border-collapse: collapse;
  }
  table,
  th,
  td {
    border: 1px solid black;
  }
</style>

<h1>Monsterbook Transcription</h1>

<form on:submit={handleTranscribe}>
  <input id="fileInput" type="file" accept="image/png" name="file[]" multiple />
  <input type="submit" value="Transcribe" />
</form>

<input type="checkbox" bind:checked={displayTable} /> Display as table {#if total > 0}
  <p>
    {#if progress < total}
      Processing {progress}/{total}
      {#if error}with {error} errors{/if} ...
    {:else}Processed {progress}/{total} with {error} errors.{/if}
  </p>
{/if}

{#if displayTable}
  <table>
    <tr>
      <th>uid</th>
      <th>name</th>
      <th>collected</th>
    </tr>
    {#each files as file}
      {#each file.data as datum}
        <tr>
          <td>{datum.uid}</td>
          <td>{datum.name}</td>
          <td>{datum.count}</td>
        </tr>
      {/each}
    {/each}
  </table>
{:else}
  <div class="wrapper">
    {#each files as file}
      {#each file.cards as card, i}
        <div>
          <img src={card} />
          <br /> uid: {file.data[i].uid}
          <br /> name: {file.data[i].name}
          <br /> count: {file.data[i].count}
          <br />
        </div>
      {/each}
    {/each}
  </div>
{/if}
