import {
    createCancelMessage,
    createRunMessage,
    MESSAGE_TYPES,
} from "./messages.js";
import { fetchGitHubRepoInputs } from "./ingest.js";
import { isProtocolMessage } from "./runtime.js";

const repoInput = document.querySelector("[data-repo]");
const refInput = document.querySelector("[data-ref]");
const tokenInput = document.querySelector("[data-token]");
const modeInput = document.querySelector("[data-mode]");
const argsInput = document.querySelector("[data-args]");
const loadRepoButton = document.querySelector("[data-load-repo]");
const cancelLoadButton = document.querySelector("[data-cancel-load]");
const runButton = document.querySelector("[data-run]");
const cancelButton = document.querySelector("[data-cancel]");
const downloadButton = document.querySelector("[data-download]");
const loadStatusOutput = document.querySelector("[data-load-status]");
const runStatusOutput = document.querySelector("[data-run-status]");
const workerCapabilitiesOutput = document.querySelector("[data-worker-capabilities]");
const repoCapabilitiesOutput = document.querySelector("[data-repo-capabilities]");
const ingestSummaryOutput = document.querySelector("[data-ingest-summary]");
const loadProgressPanel = document.querySelector("[data-load-progress-panel]");
const loadProgressElement = document.querySelector("[data-load-progress]");
const loadProgressText = document.querySelector("[data-load-progress-text]");
const resultOutput = document.querySelector("[data-result]");
const logOutput = document.querySelector("[data-log]");

const state = {
    nextRequestId: 1,
    activeRequestId: null,
    repoLoadAbortController: null,
    downloadUrl: null,
    latestResult: null,
    latestSource: null,
    latestIngest: null,
    latestLoadError: null,
    capabilities: {
        cancel: false,
        downloads: false,
        wasm: false,
        progress: false,
        zipball: false,
        modes: [],
        analyzePresets: [],
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

function formatBytes(value) {
    if (!Number.isFinite(value) || value < 0) {
        return "n/a";
    }

    if (value < 1024) {
        return `${value} B`;
    }

    if (value < 1024 * 1024) {
        return `${(value / 1024).toFixed(1)} KiB`;
    }

    return `${(value / (1024 * 1024)).toFixed(1)} MiB`;
}

function setStatus(output, message, tone = "neutral") {
    output.textContent = message;
    output.dataset.tone = tone;
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

function currentArgsOrSample(mode) {
    try {
        const parsed = JSON.parse(argsInput.value);
        if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
            return parsed;
        }
    } catch {
        // Fall back to the canned payload for the current mode.
    }

    return sampleArgsForMode(mode);
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

function updateRepoLoadControls() {
    const loading = Boolean(state.repoLoadAbortController);
    loadRepoButton.disabled = loading;
    cancelLoadButton.disabled = !loading;
    repoInput.disabled = loading;
    refInput.disabled = loading;
    tokenInput.disabled = loading;
}

function artifactFileName(data) {
    if (!data || typeof data !== "object") {
        return "tokmd-result.json";
    }

    if (data.mode === "analysis") {
        const preset = data.args?.preset ?? data.preset ?? "receipt";
        return `tokmd-analysis-${preset}.json`;
    }

    const mode = typeof data.mode === "string" ? data.mode : "result";
    return `tokmd-${mode}.json`;
}

function renderWorkerCapabilities(message) {
    const { capabilities = {}, engine = null } = message;
    const lines = [
        `engine.version: ${engine?.version ?? "unknown"}`,
        `engine.schemaVersion: ${engine?.schemaVersion ?? "n/a"}`,
        `engine.analysisSchemaVersion: ${engine?.analysisSchemaVersion ?? "n/a"}`,
        `modes: ${(capabilities.modes ?? []).join(", ")}`,
        `analyzePresets: ${(capabilities.analyzePresets ?? []).join(", ")}`,
        `wasm: ${capabilities.wasm ? "yes" : "no"}`,
        `downloads: ${capabilities.downloads ? "yes" : "no"}`,
        `runProgress: ${capabilities.progress ? "yes" : "no"}`,
        `runCancel: ${capabilities.cancel ? "yes" : "no"}`,
        `zipball: ${capabilities.zipball ? "yes" : "no"}`,
    ];
    workerCapabilitiesOutput.textContent = lines.join("\n");
}

function renderRepoCapabilities() {
    const lastAuthMode = state.latestIngest?.authMode ?? "anonymous";
    const lastCache = state.latestIngest?.cache?.hit
        ? "memory hit"
        : state.latestIngest
          ? "memory miss"
          : "not loaded yet";
    const lines = [
        "strategy: GitHub tree + contents",
        "tokenAuth: optional",
        "repoLoadProgress: yes",
        "repoLoadCancel: yes",
        "runCancel: no (worker protocol reserved only)",
        "cache: in-memory",
        "partialWarnings: surfaced",
        "rateLimitErrors: explicit",
        "zipball: no",
        `lastAuthMode: ${lastAuthMode}`,
        `lastCache: ${lastCache}`,
    ];
    repoCapabilitiesOutput.textContent = lines.join("\n");
}

function createSummaryItem(label, value) {
    const item = document.createElement("div");
    item.className = "summary-item";

    const heading = document.createElement("strong");
    heading.className = "summary-label";
    heading.textContent = label;

    const content = document.createElement("span");
    content.className = "summary-value";
    content.textContent = value;

    item.append(heading, content);
    return item;
}

function createNotice(tone, title, lines) {
    const notice = document.createElement("section");
    notice.className = `notice tone-${tone}`;

    const heading = document.createElement("strong");
    heading.textContent = title;
    notice.append(heading);

    for (const line of lines) {
        const paragraph = document.createElement("p");
        paragraph.textContent = line;
        notice.append(paragraph);
    }

    return notice;
}

function sanitizeErrorForLog(error) {
    if (!(error instanceof Error) && !(error && typeof error === "object" && typeof error.message === "string")) {
        return {
            code: "unknown",
            message: String(error),
        };
    }

    return {
        name: error.name,
        code: error.code ?? "unknown",
        message: error.message,
        status: error.status ?? null,
        resetAt: error.resetAt ?? null,
        retryAfterSeconds: error.retryAfterSeconds ?? null,
        responseMessage: error.responseMessage ?? null,
        ingest: error.ingest ?? null,
    };
}

function describeLoadError(error) {
    if (!(error instanceof Error) && !(error && typeof error === "object" && typeof error.message === "string")) {
        return String(error);
    }

    const detail = [];
    if (error.resetAt) {
        detail.push(`reset ${error.resetAt}`);
    }
    if (error.retryAfterSeconds !== undefined && error.retryAfterSeconds !== null) {
        detail.push(`retry after ${error.retryAfterSeconds}s`);
    }

    return detail.length > 0
        ? `${error.message} (${detail.join(", ")})`
        : error.message;
}

function renderLoadProgress(update = null) {
    if (!update) {
        loadProgressPanel.hidden = true;
        loadProgressElement.removeAttribute("value");
        loadProgressElement.max = 1;
        loadProgressText.textContent = "";
        return;
    }

    const total = Number.isFinite(update.total) && update.total > 0 ? update.total : 1;
    const current = Number.isFinite(update.current) ? Math.max(0, update.current) : 0;

    loadProgressPanel.hidden = false;
    loadProgressElement.max = total;
    loadProgressElement.value = Math.min(current, total);
    loadProgressText.textContent = update.message ?? `${current}/${total}`;
}

function renderIngestSummary() {
    ingestSummaryOutput.replaceChildren();

    if (!state.latestSource && !state.latestIngest && !state.latestLoadError) {
        const empty = document.createElement("p");
        empty.className = "summary-empty";
        empty.textContent = "No GitHub repo has been loaded yet.";
        ingestSummaryOutput.append(empty);
        return;
    }

    if (state.latestLoadError) {
        ingestSummaryOutput.append(
            createNotice("error", "Latest repo load error", [describeLoadError(state.latestLoadError)])
        );
    }

    if (state.latestIngest?.partialReasons?.length) {
        ingestSummaryOutput.append(
            createNotice(
                "warning",
                "Partial repo load",
                state.latestIngest.partialReasons.map((reason) => reason.message)
            )
        );
    }

    const grid = document.createElement("div");
    grid.className = "summary-grid";

    if (state.latestSource) {
        grid.append(
            createSummaryItem("Repo", state.latestSource.repo ?? "unknown"),
            createSummaryItem("Ref", state.latestSource.ref ?? "unknown"),
            createSummaryItem("Strategy", state.latestSource.strategy ?? "unknown")
        );
    }

    if (state.latestIngest) {
        grid.append(
            createSummaryItem("Auth", state.latestIngest.authMode ?? "anonymous"),
            createSummaryItem(
                "Cache",
                state.latestIngest.cache?.hit ? "memory hit" : "memory miss"
            ),
            createSummaryItem("Loaded Files", String(state.latestIngest.loadedFiles ?? 0)),
            createSummaryItem("Bytes Read", formatBytes(state.latestIngest.bytesRead ?? 0)),
            createSummaryItem("Tree Entries", String(state.latestIngest.treeEntries ?? 0)),
            createSummaryItem("Binary Skips", String(state.latestIngest.skippedBinaryContent ?? 0)),
            createSummaryItem("Vendor Skips", String(state.latestIngest.skippedVendor ?? 0)),
            createSummaryItem("Path Skips", String(state.latestIngest.skippedBinaryPath ?? 0)),
            createSummaryItem("Large File Skips", String(state.latestIngest.skippedTooLarge ?? 0)),
            createSummaryItem("Byte Budget Skips", String(state.latestIngest.skippedBudget ?? 0)),
            createSummaryItem("File Limit Skips", String(state.latestIngest.skippedFileLimit ?? 0)),
            createSummaryItem(
                "Tree Truncated",
                state.latestIngest.treeEntriesTruncated ? "yes" : "no"
            )
        );
    }

    ingestSummaryOutput.append(grid);
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
    renderWorkerCapabilities(message);
    renderRepoCapabilities();
    updateDownloadButtonState();
    cancelButton.disabled = true;
}

worker.addEventListener("message", (event) => {
    const message = event.data;
    appendLog("worker -> main", message);

    if (!isProtocolMessage(message)) {
        setStatus(runStatusOutput, "received a non-protocol worker message", "error");
        return;
    }

    switch (message.type) {
        case MESSAGE_TYPES.READY:
            setCapabilities(message);
            setStatus(
                runStatusOutput,
                message.engine?.version
                    ? `worker ready with tokmd-wasm ${message.engine.version}`
                    : "worker ready",
                "success"
            );
            break;
        case MESSAGE_TYPES.RESULT:
            if (state.activeRequestId === message.requestId) {
                state.activeRequestId = null;
            }
            cancelButton.disabled = true;
            renderLatestResult(message.data);
            setStatus(runStatusOutput, `completed ${message.requestId}`, "success");
            break;
        case MESSAGE_TYPES.ERROR:
            if (state.activeRequestId === message.requestId) {
                state.activeRequestId = null;
            }
            cancelButton.disabled = true;
            setStatus(
                runStatusOutput,
                `${message.error.code}: ${message.error.message}`,
                "error"
            );
            break;
        default:
            setStatus(runStatusOutput, `received ${message.type}`, "warning");
            break;
    }
});

worker.addEventListener("error", (event) => {
    setStatus(runStatusOutput, `worker boot failed: ${event.message}`, "error");
});

loadRepoButton.addEventListener("click", async () => {
    const repo = repoInput.value.trim();
    const ref = refInput.value.trim() || "main";
    const controller = new AbortController();

    state.repoLoadAbortController = controller;
    state.latestLoadError = null;
    updateRepoLoadControls();
    renderLoadProgress({
        phase: "start",
        current: 0,
        total: 1,
        message: `Starting browser-safe load for ${repo}@${ref}`,
    });
    setStatus(loadStatusOutput, `loading ${repo}@${ref} from GitHub...`, "working");

    try {
        const result = await fetchGitHubRepoInputs({
            repo,
            ref,
            token: tokenInput.value,
            signal: controller.signal,
            onProgress: (update) => {
                if (state.repoLoadAbortController !== controller) {
                    return;
                }

                renderLoadProgress(update);
                setStatus(loadStatusOutput, update.message, "working");
            },
        });
        const nextArgs = {
            ...currentArgsOrSample(modeInput.value),
            inputs: result.inputs,
        };

        if (
            modeInput.value === "analyze" &&
            typeof nextArgs.preset !== "string" &&
            typeof nextArgs.analyze?.preset !== "string"
        ) {
            nextArgs.preset = "estimate";
        }

        state.latestSource = result.source;
        state.latestIngest = result.ingest;
        state.latestLoadError = null;
        renderRepoCapabilities();
        renderIngestSummary();
        argsInput.value = JSON.stringify(nextArgs, null, 2);
        appendLog("github -> main", {
            source: result.source,
            ingest: result.ingest,
            samplePaths: result.inputs.slice(0, 5).map((input) => input.path),
        });
        setStatus(
            loadStatusOutput,
            result.ingest.partial
                ? `loaded ${result.ingest.loadedFiles} file(s) from ${result.source.repo}@${result.source.ref} with warnings`
                : `loaded ${result.ingest.loadedFiles} file(s) from ${result.source.repo}@${result.source.ref}`,
            result.ingest.partial ? "warning" : "success"
        );
    } catch (error) {
        const repoError = error instanceof Error ? error : (error && typeof error === "object" && error.message ? error : new Error(String(error)));
        if (repoError.ingest) {
            state.latestSource = {
                repo,
                ref,
                strategy: "github-tree-contents",
            };
            state.latestIngest = repoError.ingest;
        }
        state.latestLoadError = repoError;
        renderRepoCapabilities();
        renderIngestSummary();
        appendLog("github error -> main", sanitizeErrorForLog(repoError));
        renderLoadProgress({
            phase: repoError.name === "AbortError" ? "aborted" : "error",
            current: 0,
            total: 1,
            message: describeLoadError(repoError),
        });
        setStatus(
            loadStatusOutput,
            repoError.name === "AbortError"
                ? "repo load canceled"
                : `repo load failed: ${describeLoadError(repoError)}`,
            repoError.name === "AbortError" ? "warning" : "error"
        );
    } finally {
        if (state.repoLoadAbortController === controller) {
            state.repoLoadAbortController = null;
        }
        updateRepoLoadControls();
    }
});

cancelLoadButton.addEventListener("click", () => {
    if (!state.repoLoadAbortController) {
        setStatus(loadStatusOutput, "no active repo load", "warning");
        return;
    }

    state.repoLoadAbortController.abort();
    setStatus(loadStatusOutput, "canceling repo load...", "warning");
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
        setStatus(runStatusOutput, `invalid JSON: ${error.message}`, "error");
        return;
    }

    const requestId = `run-${state.nextRequestId++}`;
    state.activeRequestId = requestId;
    cancelButton.disabled = !state.capabilities.cancel;
    setStatus(runStatusOutput, `sent ${requestId}`, "working");

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
        setStatus(runStatusOutput, "no active request", "warning");
        return;
    }

    const message = createCancelMessage(state.activeRequestId);
    appendLog("main -> worker", message);
    worker.postMessage(message);
});

downloadButton.addEventListener("click", () => {
    if (!state.downloadUrl || !state.latestResult) {
        setStatus(runStatusOutput, "no result to download yet", "warning");
        return;
    }

    const link = document.createElement("a");
    link.href = state.downloadUrl;
    link.download = downloadButton.dataset.filename || "tokmd-result.json";
    link.click();
    setStatus(runStatusOutput, `downloaded ${link.download}`, "success");
});

setStatus(loadStatusOutput, "repo load idle", "neutral");
setStatus(runStatusOutput, "starting worker...", "neutral");
renderRepoCapabilities();
renderIngestSummary();
updateRepoLoadControls();
setSampleArgs(modeInput.value);
