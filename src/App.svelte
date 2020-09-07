<script>
  import { reference, empty, seed_tags, entries } from "./assets.js";
  import {
    readImageAsync,
    canvasFromImage,
    cloneCanvas,
    sobel,
    rgb2gray,
  } from "./image.js";
  import { onMount } from "svelte";
  import { claim_text } from "svelte/internal";

  let canvas;

  onMount(async () => {
    canvas = await canvasFromImage(reference[0]);
    sobel(rgb2gray(canvas));
  });
</script>

<h1>Monsterbook Transcription</h1>

{#if canvas}<img src={canvas.toDataURL()} />{/if}
<img src={reference[0]} />
<img src={empty} />
<img src={seed_tags[0]} />
<pre>{entries}</pre>
