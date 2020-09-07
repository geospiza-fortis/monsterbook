<script>
  import { onMount } from "svelte";
  import { transcribe } from "./book.js";
  import { readDataAsync } from "./image.js";

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
          <img src={card} alt={file.data[i].name} />
          <br /> uid: {file.data[i].uid}
          <br /> name: {file.data[i].name}
          <br /> count: {file.data[i].count}
          <br />
        </div>
      {/each}
    {/each}
  </div>
{/if}
