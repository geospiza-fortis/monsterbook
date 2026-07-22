// Web worker owning the Session. All wasm (mock, for now) work happens here
// so the main thread never blocks.
//
// WASM SWAP POINT: replace the import below with the wasm-pack module, e.g.
//   import init, { Session } from "monsterbook-wasm";
//   await init();
import { Session } from "./mock-session.js";

const session = new Session();

// Protocol: request { id, op, ...args } -> response { id, ok, result | error }.
// Image buffers are passed as transferables in both directions.
self.onmessage = async (event) => {
  const { id, op } = event.data;
  try {
    let result;
    let transfer = [];
    switch (op) {
      case "add_screenshot": {
        const bytes = new Uint8Array(event.data.buffer);
        result = session.add_screenshot(bytes);
        break;
      }
      case "add_bitmap": {
        const { width, height } = event.data;
        const rgba = new Uint8Array(event.data.buffer);
        result = session.add_bitmap(width, height, rgba);
        break;
      }
      case "missing":
        result = session.missing();
        break;
      case "transcribe":
        result = session.transcribe();
        break;
      case "stitch": {
        const png = await session.stitch(event.data.cardsPerRow);
        result = png.buffer;
        transfer = [png.buffer];
        break;
      }
      default:
        throw { kind: "UnknownOp" };
    }
    self.postMessage({ id, ok: true, result }, transfer);
  } catch (error) {
    self.postMessage({
      id,
      ok: false,
      error: { kind: error?.kind ?? "InternalError", message: error?.message },
    });
  }
};
