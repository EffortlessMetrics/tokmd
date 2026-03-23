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
const downloadButton = document.querySelector("[data-download]");
const statusOutput = document.querySelector("[data-status]");
const capabilitiesOutput = document.querySelector("[data-capabilities]");
const resultOutput = document.querySelector("[data-result]");
const logOutput = document.querySelector("[data-log]");

const state = {
    nextRequestId: 1,
    activeRequestId: null,
    downloadUrl: null,
    latestResult: null,
    capabilities: {
        cancel: false,
        downloads: false,
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
        case "export":
            return {
                inputs: sampleInputs(),
            };
        case "analyze":
            return {
                inputs: sampleInputs(),
                preset: "estimate",
            };
        default:
            return {
                inputs: sampleInputs(),
            };
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

function clearDownloadUrl() {
    if (state.downloadUrl) {
        URL.revokeObjectURL(state.downloadUrl);
        state.downloadUrl = null;
    }

    delete downloadButton.dataset.filename;
}

function updateDownloadButtonState() {
    downloadButton.disabled = !(
        state.capabilities.downloads &&
        state.downloadUrl &&
        state.latestResult
    );
}

function artifactFileName(data) {
    if (!data || typeof data !== "object") {
        return "tokmd-result.json";
    }

    if (data.mode === "analysis") {
        const preset = data.args?.preset ?? "receipt";
        return `tokmd-analysis-${preset}.json`;
    }

    const mode = typeof data.mode === "string" ? data.mode : "result";
    return `tokmd-${mode}.json`;
}

function renderCapabilities(message) {
    const { capabilities = {}, engine = null } = message;
    const lines = [
        `engine.version: ${engine?.version ?? "unknown"}`,
        `engine.schemaVersion: ${engine?.schemaVersion ?? "n/a"}`,
        `engine.analysisSchemaVersion: ${engine?.analysisSchemaVersion ?? "n/a"}`,
        `modes: ${(capabilities.modes ?? []).join(", ")}`,
        `analyzePresets: ${(capabilities.analyzePresets ?? []).join(", ")}`,
        `wasm: ${capabilities.wasm ? "yes" : "no"}`,
        `downloads: ${capabilities.downloads ? "yes" : "no"}`,
        `zipball: ${capabilities.zipball ? "yes" : "no"}`,
        `progress: ${capabilities.progress ? "yes" : "no"}`,
        `cancel: ${capabilities.cancel ? "yes" : "no"}`,
    ];
    capabilitiesOutput.textContent = lines.join("\n");
}

function renderLatestResult(data) {
    state.latestResult = data;
    clearDownloadUrl();
    resultOutput.textContent = JSON.stringify(data, null, 2);

    if (!state.capabilities.downloads) {
        updateDownloadButtonState();
        return;
    }

    const blob = new Blob([`${JSON.stringify(data, null, 2)}\n`], {
        type: "application/json",
    });
    state.downloadUrl = URL.createObjectURL(blob);
    downloadButton.dataset.filename = artifactFileName(data);
    updateDownloadButtonState();
}

function setCapabilities(message) {
    state.capabilities = {
        ...message.capabilities,
    };
    renderCapabilities(message);
    updateDownloadButtonState();
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
            renderStatus(
                message.engine?.version
                    ? `worker ready with tokmd-wasm ${message.engine.version}`
                    : "worker ready"
            );
            break;
        case MESSAGE_TYPES.RESULT:
            if (state.activeRequestId === message.requestId) {
                state.activeRequestId = null;
            }
            cancelButton.disabled = true;
            renderLatestResult(message.data);
            renderStatus(`completed ${message.requestId}`);
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

window.addEventListener("beforeunload", () => {
    clearDownloadUrl();
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

downloadButton.addEventListener("click", () => {
    if (!state.downloadUrl || !state.latestResult) {
        renderStatus("no result to download yet");
        return;
    }

    const link = document.createElement("a");
    link.href = state.downloadUrl;
    link.download = downloadButton.dataset.filename || "tokmd-result.json";
    link.click();
    renderStatus(`downloaded ${link.download}`);
});

renderStatus("starting worker...");
setSampleArgs(modeInput.value);
