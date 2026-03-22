import { createReadyMessage } from "./messages.js";
import { handleRunnerMessage } from "./runtime.js";

globalThis.postMessage(createReadyMessage());

globalThis.addEventListener("message", (event) => {
    globalThis.postMessage(handleRunnerMessage(event.data));
});
