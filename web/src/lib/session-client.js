// Promise-based RPC wrapper around the session worker.

let worker;
let nextId = 0;
const pending = new Map();

function getWorker() {
  if (!worker) {
    worker = new Worker(new URL("./worker.js", import.meta.url), {
      type: "module",
    });
    worker.onmessage = (event) => {
      const { id, ok, result, error } = event.data;
      const p = pending.get(id);
      if (!p) return;
      pending.delete(id);
      ok ? p.resolve(result) : p.reject(error);
    };
  }
  return worker;
}

function call(message, transfer = []) {
  const id = nextId++;
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject });
    getWorker().postMessage({ id, ...message }, transfer);
  });
}

export const addScreenshot = (buffer) =>
  call({ op: "add_screenshot", buffer }, [buffer]);
export const addBitmap = (width, height, buffer) =>
  call({ op: "add_bitmap", width, height, buffer }, [buffer]);
export const missing = () => call({ op: "missing" });
export const transcribe = () => call({ op: "transcribe" });
export const stitch = (cardsPerRow, includeEmpty = false) =>
  call({ op: "stitch", cardsPerRow, includeEmpty });
