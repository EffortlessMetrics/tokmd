import { createReadyMessage } from "./messages.js";
import { handleRunnerMessage } from "./runtime.js";

let emitMessage;
let subscribe;

if (
    typeof globalThis.postMessage === "function" &&
    typeof globalThis.addEventListener === "function"
) {
    emitMessage = (message) => globalThis.postMessage(message);
    subscribe = (handler) => {
        globalThis.addEventListener("message", (event) => {
            handler(event.data);
        });
    };
} else {
    const { parentPort } = await import("node:worker_threads");

    emitMessage = (message) => parentPort.postMessage(message);
    subscribe = (handler) => {
        parentPort.on("message", handler);
    };
}

emitMessage(createReadyMessage());
subscribe((message) => {
    emitMessage(handleRunnerMessage(message));
});
