import convolve from "convolve";

// https://simon-schraeder.de/posts/filereader-async/
async function readDataAsync(file) {
  return new Promise((resolve, reject) => {
    let reader = new FileReader();
    reader.onload = () => resolve(reader.result);
    reader.onerror = reject;
    reader.readAsDataURL(file);
  });
}

async function readImageAsync(dataUrl) {
  return new Promise((resolve, _) => {
    let img = new Image();
    img.onload = () => {
      resolve(img);
    };
    img.src = dataUrl;
  });
}

function canvasFromImage(dataUrl) {
  return new Promise((resolve, _) => {
    let canvas = document.createElement("canvas");
    let context = canvas.getContext("2d");
    let img = new Image();
    img.onload = () => {
      canvas.width = img.width;
      canvas.height = img.height;
      context.drawImage(img, 0, 0);
      resolve(canvas);
    };
    img.src = dataUrl;
  });
}

// https://stackoverflow.com/a/8306028
function cloneCanvas(canvas) {
  let clone = document.createElement("canvas");
  let context = clone.getContext("2d");
  clone.width = canvas.width;
  clone.height = canvas.height;
  context.drawImage(canvas, 0, 0);
  return clone;
}

async function cropImage(dataUrl, y0, y1, x0, x1) {
  let srcCanvas = await canvasFromImage(dataUrl);
  let destCanvas = document.createElement("canvas");
  let width = x1 - x0;
  let height = y1 - y0;
  destCanvas.width = width;
  destCanvas.height = height;
  let destContext = destCanvas.getContext("2d");

  destContext.drawImage(srcCanvas, x0, y0, width, height, 0, 0, width, height);
  return destCanvas.toDataURL();
}

function rgb2gray(canvas) {
  // https://stackoverflow.com/a/37160699
  function luma(data, i) {
    return data[i] * 0.2989 + data[i + 1] * 0.2989 + data[i + 2] * 0.114;
  }
  let ctx = canvas.getContext("2d");
  let data = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
  let gray = ctx.createImageData(canvas.width, canvas.height);
  for (let i = 0; i < data.length; i += 4) {
    // all channels are the same now, with exception of alpha
    gray.data[i] = gray.data[i + 1] = gray.data[i + 2] = luma(data, i);
    gray.data[i + 3] = data[i + 3];
  }
  ctx.putImageData(gray, 0, 0);
  return canvas;
}

// filters the canvas in place
function sobel(canvas) {
  let Gx = [
    [1, 0, -1],
    [2, 0, -2],
    [1, 0, -1],
  ];
  let Gy = [
    [1, 2, 1],
    [0, 0, 0],
    [-1, -2, -1],
  ];
  let horizontal = cloneCanvas(canvas);
  convolve(Gx).canvas(canvas);
  convolve(Gy).canvas(horizontal);
  // update the vertical canvas in place and return it
  let ctx = canvas.getContext("2d");
  let image = ctx.getImageData(0, 0, canvas.width, canvas.height);
  let data = image.data;
  let horizontalData = horizontal
    .getContext("2d")
    .getImageData(0, 0, canvas.width, canvas.height).data;

  for (let i = 0; i < data.length; i++) {
    data[i] += horizontalData[i];
  }
  ctx.putImageData(image, 0, 0);
  return canvas;
}

// this will cause the aspect ratio to be funky if the target is not the same
async function resizeImage(dataUrl, width, height) {
  let srcCanvas = await canvasFromImage(dataUrl);
  let destCanvas = document.createElement("canvas");
  let destContext = destCanvas.getContext("2d");
  destCanvas.width = width;
  destCanvas.height = height;
  destContext.drawImage(srcCanvas, 0, 0, width, height);
  return destCanvas.toDataURL();
}

export {
  cropImage,
  resizeImage,
  readDataAsync,
  readImageAsync,
  canvasFromImage,
  cloneCanvas,
  rgb2gray,
  sobel,
};
