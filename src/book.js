import { reference, empty, seed_tags, entries } from "./assets.js";
import { metadata, offsets } from "./metadata.js";
import {
  readImageAsync,
  canvasFromImage,
  sobel,
  rgb2gray,
  cropImage,
  match,
  mse,
} from "./image.js";

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
  let index = await match(img, reference, (canvas) => sobel(rgb2gray(canvas)));

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

export { transcribe };
