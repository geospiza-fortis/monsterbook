import { reference, empty, seed_tags } from "./assets.js";
import { monsters } from "./monsters.js";
import { metadata, offsets } from "./metadata.js";
import {
  readImageAsync,
  canvasFromImage,
  sobel,
  rgb2gray,
  cropImage,
  match,
  mse,
  resizeImage,
} from "./image.js";

const CROP_HEIGHT = 225;
const CROP_WIDTH = 165;
const TAG_HEIGHT = 9;
const TAG_WIDTH = 6;

function is_windows(height, width) {
  return (
    (height == 600 && width == 800) ||
    (height == 768 && width == 1024) ||
    (height == 768 && width == 1366)
  );
}

async function crop(dataUrl) {
  let img = await readImageAsync(dataUrl);
  if (!is_windows(img.height, img.width)) {
    // NOTE: this does not cover non-retina, HDClient=1
    let x = 412;
    let y = 862;
    let h = CROP_HEIGHT * 2;
    let w = CROP_WIDTH * 2;
    let img = await cropImage(dataUrl, x, x + h, y, y + w);
    return await resizeImage(img, CROP_WIDTH, CROP_HEIGHT);
  } else {
    let x = 152;
    let y = 295;
    let h = CROP_HEIGHT;
    let w = CROP_WIDTH;
    return await cropImage(dataUrl, x, x + h, y, y + w);
  }
}

async function crop_tag(dataUrl) {
  let x = 31;
  let y = 5;
  let h = TAG_HEIGHT;
  let w = TAG_WIDTH;
  return await cropImage(dataUrl, x, x + h, y, y + w);
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
  // TODO: determine these thresholds
  const empty_threshold = 100;
  const unseen_threshold = 5000;

  // TODO: fix this by resizing all of the assets to match the new sizes because
  // this is wasteful.
  let reference_resized = await Promise.all(
    reference.map(async (img) => {
      return await resizeImage(img, CROP_WIDTH, CROP_HEIGHT);
    })
  );

  let empty_resized = await resizeImage(empty, CROP_WIDTH / 5, CROP_HEIGHT / 5);

  let seed_tags_resized = await Promise.all(
    seed_tags.map(async (img) => {
      return await resizeImage(img, TAG_WIDTH, TAG_HEIGHT);
    })
  );

  let img = await crop(dataUrl);

  // match this against one of the existing files
  let index = await match(img, reference_resized, (canvas) =>
    sobel(rgb2gray(canvas))
  );

  let cards = await crop_cards(img);
  // TODO: what is the correct threshold here? I don't want to pull in a new dependency
  // to rigorously determine the threshold.
  let filtered = await filter_cards(cards, empty_threshold);

  // determine the appropriate tag for each one
  let cardData = [];
  let emptyCanvas = await canvasFromImage(empty_resized);
  for (let i = 0; i < filtered.length; i++) {
    let tag = await crop_tag(filtered[i]);
    let value = 0;
    let err = mse(await canvasFromImage(tag), emptyCanvas, rgb2gray);
    // console.log(err);
    if (err > unseen_threshold) {
      value = (await match(tag, seed_tags_resized)) + 1;
    }
    let data = {
      img: filtered[i],
      uid: offsets[index] + i,
      count: value,
      ...monsters[offsets[index] + i],
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
