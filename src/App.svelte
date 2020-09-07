<script>
  import { onMount } from "svelte";
  import { transcribe } from "./book.js";
  import { readDataAsync } from "./image.js";

  let displayTable = true;
  let displayDone = false;
  let displaySort = "uid";

  const displaySortOptions = ["uid", "tour_order", "collected"];
  const displaySortFunc = {
    uid: (a, b) => a.uid - b.uid,
    tour_order: (a, b) => a.tour_order - b.tour_order || a.uid - b.uid,
    collected: (a, b) =>
      b.count - a.count || a.tour_order - b.tour_order || a.uid - b.uid,
  };

  let files = [];
  let progress = 0;
  let total = 0;
  let error = 0;
  $: data = flattenData(files, displaySort);

  function flattenData(files, displaySort) {
    let res = [];
    for (let i = 0; i < files.length; i++) {
      res = res.concat(files[i].data);
    }
    res.sort(displaySortFunc[displaySort]);
    return res;
  }

  async function appendToFiles(filelist) {
    error = 0;
    total = filelist.length;
    for (let i = 0; i < filelist.length; i++) {
      progress = i;
      let file = filelist[i];
      let dataUrl = await readDataAsync(file);
      let transcribed = await transcribe(dataUrl);
      // TODO: handle duplicates
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

{#if total > 0}
  <p>
    {#if progress < total}
      Processing {progress}/{total}
      {#if error}with {error} errors{/if} ...
    {:else}Processed {progress}/{total} with {error} errors.{/if}
  </p>
{/if}

<label><input type="checkbox" bind:checked={displayTable} /> Display as table</label>
<label><input type="checkbox" bind:checked={displayDone} /> Display finished sets</label>
Default Sort <br />
{#each displaySortOptions as value}
  <label><input type="radio" {value} bind:group={displaySort} />{value}</label>
{/each}
{#if displayTable}
  {#if data}
    <table>
      <tr>
        <th>uid</th>
        <th>name</th>
        <th>collected</th>
        <th>most efficient map</th>
        <th>town</th>
        <th>tour order</th>
      </tr>
      {#each data as datum}
        {#if !(!displayDone && datum.count === 5)}
          <tr>
            <td>{datum.uid}</td>
            <td>{datum.name}</td>
            <td>{datum.count}</td>
            <td>{datum.map}</td>
            <td>{datum.town}</td>
            <td>{datum.tour_order}</td>
          </tr>
        {/if}
      {/each}
    </table>
  {/if}
{:else}
  <div class="wrapper">
    {#each files as file}
      {#each file.cards as card, i}
        {#if !(!displayDone && file.data[i].count === 5)}
          <div>
            <img src={card} alt={file.data[i].name} />
            <br /> uid: {file.data[i].uid}
            <br /> name: {file.data[i].name}
            <br /> count: {file.data[i].count}
            <br />
          </div>
        {/if}
      {/each}
    {/each}
  </div>
{/if}
