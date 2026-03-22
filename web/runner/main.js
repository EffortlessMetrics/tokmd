import {
    createCancelMessage,
    createRunMessage,
    MESSAGE_TYPES,
} from "./messages.js";
import { isProtocolMessage } from "./runtime.js";

const modeInput = document.querySelector("[data-mode]");
const argsInput = document.querySelector("[data-args]");
const runButton = document.querySelector("[data-run]");
const cancelButton = document.querySelector("[data-cancel]");
const statusOutput = document.querySelector("[data-status]");
const capabilitiesOutput = document.querySelector("[data-capabilities]");
const logOutput = document.querySelector("[data-log]");

const state = {
    nextRequestId: 1,
    activeRequestId: null,
    capabilities: {
        cancel: false,
    },
};

const worker = new Worker(new URL("./worker.js", import.meta.url), {
    type: "module",
});

function sampleInputs() {
    return [
        {
            path: "src/lib.rs",
            text: "pub fn alpha() -> usize { 1 }\n",
        },
        {
            path: "tests/basic.py",
            text: "print('ok')\n",
        },
    ];
}

function sampleArgsForMode(mode) {
    switch (mode) {
        case "lang":
            return {
                inputs: sampleInputs(),
                files: true,
            };
        case "module":
            return {
                inputs: sampleInputs(),
            };
        case "export":
            return {
                inputs: sampleInputs(),
            };
        case "analyze":
            return {
                inputs: sampleInputs(),
            };
        default:
            return { inputs: sampleInputs() };
    }
}

function renderStatus(message) {
    statusOutput.textContent = message;
}

function appendLog(label, payload) {
    const block = document.createElement("pre");
    block.className = "log-entry";
    block.textContent = `${label}\n${JSON.stringify(payload, null, 2)}`;
    logOutput.prepend(block);
}

function setSampleArgs(mode) {
    argsInput.value = JSON.stringify(sampleArgsForMode(mode), null, 2);
}

function setCapabilities(message) {
    state.capabilities = {
        ...message.capabilities,
    };
    capabilitiesOutput.textContent = JSON.stringify(
        message.capabilities,
        null,
        2
    );
    cancelButton.disabled = true;
}

worker.addEventListener("message", (event) => {
    const message = event.data;
    appendLog("worker -> main", message);

    if (!isProtocolMessage(message)) {
        renderStatus("received a non-protocol worker message");
        return;
    }

    switch (message.type) {
        case MESSAGE_TYPES.READY:
            setCapabilities(message);
            renderStatus("worker ready");
            break;
        case MESSAGE_TYPES.ERROR:
            if (state.activeRequestId === message.requestId) {
                state.activeRequestId = null;
            }
            cancelButton.disabled = true;
            renderStatus(`${message.error.code}: ${message.error.message}`);
            break;
        default:
            renderStatus(`received ${message.type}`);
            break;
    }
});

worker.addEventListener("error", (event) => {
    renderStatus(`worker boot failed: ${event.message}`);
});

modeInput.addEventListener("change", () => {
    setSampleArgs(modeInput.value);
});

runButton.addEventListener("click", () => {
    let args;

    try {
        args = JSON.parse(argsInput.value);
    } catch (error) {
        renderStatus(`invalid JSON: ${error.message}`);
        return;
    }

    const requestId = `run-${state.nextRequestId++}`;
    state.activeRequestId = requestId;
    cancelButton.disabled = !state.capabilities.cancel;
    renderStatus(`sent ${requestId}`);

    const message = createRunMessage({
        requestId,
        mode: modeInput.value,
        args,
    });

    appendLog("main -> worker", message);
    worker.postMessage(message);
});

cancelButton.addEventListener("click", () => {
    if (!state.activeRequestId) {
        renderStatus("no active request");
        return;
    }

    const message = createCancelMessage(state.activeRequestId);
    appendLog("main -> worker", message);
    worker.postMessage(message);
});

renderStatus("starting worker...");
setSampleArgs(modeInput.value);
