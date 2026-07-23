// Web worker owning the Session. All wasm work happens here so the main
// thread never blocks.
//
// The real wasm module (built via `npm run build:wasm`, see web/README.md)
// lives in ./pkg. Set VITE_MOCK_SESSION=1 to fall back to the mock
// implementation in mock-session.js for UI development without a wasm build.

const useMock = import.meta.env?.VITE_MOCK_SESSION === "1";

// Init handshake: messages arriving before wasm init resolves await the
// `ready` promise, so callers never race the module load.
const ready = (async () => {
  if (useMock) {
    const { Session } = await import("./mock-session.js");
    return new Session();
  }
  const wasm = await import("./pkg/monsterbook.js");
  await wasm.default(); // init()
  return new wasm.Session();
})();

// Protocol: request { id, op, ...args } -> response { id, ok, result | error }.
// Image buffers are passed as transferables in both directions.
self.onmessage = async (event) => {
  const { id, op } = event.data;
  try {
    const session = await ready;
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
        result = Array.from(session.missing());
        break;
      case "transcribe":
        result = session.transcribe();
        break;
      case "stitch": {
        const png = await session.stitch(
          event.data.cardsPerRow,
          event.data.includeEmpty,
        );
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
