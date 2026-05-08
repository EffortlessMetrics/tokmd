import {
    createCancelMessage,
    createRunMessage,
    MESSAGE_TYPES,
} from "./messages.js";
import {
    authModeForToken,
    clearSessionToken,
    readSessionToken,
    resolveSessionStorage,
    writeSessionToken,
} from "./auth.js";
import { fetchGitHubRepoInputs } from "./ingest.js";
import { isProtocolMessage } from "./runtime.js";

const repoInput = document.querySelector("[data-repo]");
const refInput = document.querySelector("[data-ref]");
const tokenInput = document.querySelector("[data-token]");
const clearTokenButton = document.querySelector("[data-clear-token]");
const authStateOutput = document.querySelector("[data-auth-state]");
const modeInput = document.querySelector("[data-mode]");
const argsInput = document.querySelector("[data-args]");
const localFilesInput = document.querySelector("[data-local-files]");
const localDirectoryInput = document.querySelector("[data-local-directory]");
const loadLocalButton = document.querySelector("[data-load-local]");
const loadRepoButton = document.querySelector("[data-load-repo]");
const retryLoadButton = document.querySelector("[data-retry-load]");
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
const runProgressPanel = document.querySelector("[data-run-progress-panel]");
const runProgressElement = document.querySelector("[data-run-progress]");
const runProgressText = document.querySelector("[data-run-progress-text]");
const resultOutput = document.querySelector("[data-result]");
const logOutput = document.querySelector("[data-log]");

const BROWSER_MODE_ORDER = ["lang", "module", "export", "analyze"];

const state = {
    nextRequestId: 1,
    activeRequestId: null,
    repoLoadAbortController: null,
    localLoadActive: false,
    downloadUrl: null,
    latestResult: null,
    latestSource: null,
    latestIngest: null,
    latestLoadError: null,
    latestAuthIssue: null,
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
                preset: defaultAnalyzePreset(),
            };
        default:
            return {
                inputs: sampleInputs(),
            };
    }
}

function defaultAnalyzePreset() {
    const presets = Array.isArray(state.capabilities.analyzePresets)
        ? state.capabilities.analyzePresets
        : [];

    return presets.includes("estimate") ? "estimate" : presets[0] ?? "receipt";
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

function argsWithInputs(inputs) {
    const nextArgs = {
        ...currentArgsOrSample(modeInput.value),
        inputs,
    };

    if (
        modeInput.value === "analyze" &&
        typeof nextArgs.preset !== "string" &&
        typeof nextArgs.analyze?.preset !== "string"
    ) {
        nextArgs.preset = defaultAnalyzePreset();
    }

    return nextArgs;
}

function normalizeLocalFilePath(file, index) {
    const raw =
        typeof file?.webkitRelativePath === "string" && file.webkitRelativePath.trim()
            ? file.webkitRelativePath
            : typeof file?.name === "string" && file.name.trim()
              ? file.name
              : `file-${index + 1}`;

    return raw.replace(/\\/g, "/").replace(/^\/+/, "") || `file-${index + 1}`;
}

function compareByCodePoint(left, right) {
    let leftIndex = 0;
    let rightIndex = 0;

    while (leftIndex < left.length && rightIndex < right.length) {
        const leftCodePoint = left.codePointAt(leftIndex);
        const rightCodePoint = right.codePointAt(rightIndex);

        if (leftCodePoint !== rightCodePoint) {
            return leftCodePoint < rightCodePoint ? -1 : 1;
        }

        leftIndex += leftCodePoint > 0xffff ? 2 : 1;
        rightIndex += rightCodePoint > 0xffff ? 2 : 1;
    }

    if (leftIndex === left.length && rightIndex === right.length) {
        return 0;
    }

    return leftIndex === left.length ? -1 : 1;
}

function localIngestReceipt(fileEntries, inputs, bytesRead) {
    return {
        bytesRead,
        loadedFiles: inputs.length,
        skippedBinaryContent: 0,
        skippedBudget: 0,
        partial: false,
        partialReasons: [],
        treeEntriesTruncated: false,
        cache: {
            scope: "none",
            hit: false,
        },
        authMode: "local",
        treeEntries: fileEntries.length,
        selectedFiles: inputs.length,
        skippedVendor: 0,
        skippedBinaryPath: 0,
        skippedTooLarge: 0,
        skippedFileLimit: 0,
        maxFiles: fileEntries.length,
        maxBytes: bytesRead,
        maxFileBytes: fileEntries.reduce(
            (largest, entry) =>
                Number.isFinite(entry.file?.size)
                    ? Math.max(largest, entry.file.size)
                    : largest,
            0
        ),
    };
}

function selectedLocalFileEntries() {
    return [
        ...Array.from(localFilesInput.files ?? []),
        ...Array.from(localDirectoryInput.files ?? []),
    ]
        .map((file, index) => ({
            file,
            path: normalizeLocalFilePath(file, index),
        }))
        .sort((left, right) => compareByCodePoint(left.path, right.path));
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

function selectableModes() {
    const advertised = Array.isArray(state.capabilities.modes)
        ? state.capabilities.modes
        : [];
    const presets = Array.isArray(state.capabilities.analyzePresets)
        ? state.capabilities.analyzePresets
        : [];

    return BROWSER_MODE_ORDER.filter(
        (mode) => advertised.includes(mode) && (mode !== "analyze" || presets.length > 0)
    );
}

function updateRunControls() {
    const selectable = new Set(selectableModes());
    const options = Array.from(modeInput.options ?? []);

    for (const option of options) {
        option.disabled = !selectable.has(option.value);
    }

    if (!selectable.has(modeInput.value)) {
        const nextMode = BROWSER_MODE_ORDER.find((mode) => selectable.has(mode));
        if (nextMode) {
            modeInput.value = nextMode;
            setSampleArgs(nextMode);
        }
    }

    runButton.disabled = selectable.size === 0;
}

function updateRepoLoadControls() {
    const repoLoading = Boolean(state.repoLoadAbortController);
    const loading = repoLoading || state.localLoadActive;
    loadLocalButton.disabled = loading;
    localFilesInput.disabled = loading;
    localDirectoryInput.disabled = loading;
    loadRepoButton.disabled = loading;
    retryLoadButton.disabled = loading || !isRetryableLoadError(state.latestLoadError);
    cancelLoadButton.disabled = !repoLoading;
    repoInput.disabled = loading;
    refInput.disabled = loading;
    tokenInput.disabled = loading;
    clearTokenButton.disabled = loading || authModeForToken(tokenInput.value) === "anonymous";
}

function isAuthRepairLoadError(error) {
    return (
        error instanceof Error &&
        (error.code === "github_auth_required" || error.code === "github_repo_unavailable")
    );
}

function renderAuthState() {
    const authMode = authModeForToken(tokenInput.value);
    if (authMode === "anonymous") {
        authStateOutput.textContent = "anonymous";
        authStateOutput.dataset.tone = "neutral";
        return;
    }

    if (state.latestAuthIssue) {
        authStateOutput.textContent =
            state.latestAuthIssue.code === "github_repo_unavailable"
                ? "check token/repo"
                : "token rejected";
        authStateOutput.dataset.tone = "error";
        return;
    }

    authStateOutput.textContent = "authenticated";
    authStateOutput.dataset.tone = "success";
}

function syncTokenState({ persist = true } = {}) {
    const storage = resolveSessionStorage();
    const token = persist
        ? writeSessionToken(storage, tokenInput.value)
        : readSessionToken(storage);

    if (!persist) {
        tokenInput.value = token;
    }

    renderAuthState();
    updateRepoLoadControls();
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
    const lastCache =
        state.latestIngest?.cache?.scope === "none"
            ? "none"
            : state.latestIngest?.cache?.hit
              ? "memory hit"
              : state.latestIngest
                ? "memory miss"
                : "not loaded yet";
    const lines = [
        "strategy: GitHub tree + contents or local files",
        "tokenAuth: optional",
        "repoLoadProgress: yes",
        "repoLoadCancel: yes",
        "localFiles: yes",
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
    if (!(error instanceof Error)) {
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
        retryAt: error.retryAt ?? null,
        responseMessage: error.responseMessage ?? null,
        ingest: error.ingest ?? null,
    };
}

function isRetryableLoadError(error) {
    return (
        error instanceof Error &&
        (error.code === "github_primary_rate_limit" ||
            error.code === "github_secondary_rate_limit")
    );
}

function describeRetryWindow(error) {
    if (!(error instanceof Error)) {
        return "";
    }

    if (error.retryAfterSeconds !== undefined && error.retryAfterSeconds !== null) {
        return `Retry after ${error.retryAfterSeconds}s.`;
    }

    if (error.retryAt) {
        return `Retry after ${error.retryAt}.`;
    }

    if (error.resetAt) {
        return `Retry after GitHub resets the quota at ${error.resetAt}.`;
    }

    return "Retry when GitHub accepts more browser repo requests.";
}

function describeLoadError(error) {
    if (!(error instanceof Error)) {
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

function loadErrorNoticeLines(error) {
    const lines = [describeLoadError(error)];
    if (isRetryableLoadError(error)) {
        lines.push(describeRetryWindow(error));
    }
    if (isAuthRepairLoadError(error)) {
        lines.push("Update or clear the GitHub token, then verify the repository and ref.");
    }

    return lines;
}

function describeWorkerProgress(message) {
    if (typeof message.message === "string" && message.message.trim()) {
        return message.message;
    }

    return message.phase ? `worker ${message.phase}` : "worker progress";
}

function workerProgressTone(phase) {
    if (phase === "done") {
        return "success";
    }

    if (phase === "error") {
        return "error";
    }

    return "working";
}

function isStaleRunScopedWorkerMessage(message) {
    return (
        typeof message.requestId === "string" &&
        state.activeRequestId !== message.requestId
    );
}

function renderProgress(target, update = null) {
    const { panel, element, text } = target;

    if (!update) {
        panel.hidden = true;
        element.removeAttribute("value");
        element.max = 1;
        text.textContent = "";
        return;
    }

    const total = Number.isFinite(update.total) && update.total > 0 ? update.total : 1;
    const current = Number.isFinite(update.current) ? Math.max(0, update.current) : 0;

    panel.hidden = false;
    element.max = total;
    element.value = Math.min(current, total);
    text.textContent = update.message ?? `${current}/${total}`;
}

function renderLoadProgress(update = null) {
    renderProgress(
        {
            panel: loadProgressPanel,
            element: loadProgressElement,
            text: loadProgressText,
        },
        update
    );
}

function renderRunProgress(update = null) {
    renderProgress(
        {
            panel: runProgressPanel,
            element: runProgressElement,
            text: runProgressText,
        },
        update
    );
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
            createNotice(
                "error",
                "Latest repo load error",
                loadErrorNoticeLines(state.latestLoadError)
            )
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
    updateRunControls();
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
        case MESSAGE_TYPES.PROGRESS:
            if (!message.requestId || state.activeRequestId === message.requestId) {
                renderRunProgress(message);
                setStatus(
                    runStatusOutput,
                    describeWorkerProgress(message),
                    workerProgressTone(message.phase)
                );
            }
            break;
        case MESSAGE_TYPES.RESULT:
            if (isStaleRunScopedWorkerMessage(message)) {
                break;
            }
            if (state.activeRequestId === message.requestId) {
                state.activeRequestId = null;
            }
            cancelButton.disabled = true;
            renderRunProgress({
                phase: "done",
                current: 1,
                total: 1,
                message: `completed ${message.requestId}`,
            });
            renderLatestResult(message.data);
            setStatus(runStatusOutput, `completed ${message.requestId}`, "success");
            break;
        case MESSAGE_TYPES.ERROR:
            if (isStaleRunScopedWorkerMessage(message)) {
                break;
            }
            if (state.activeRequestId === message.requestId) {
                state.activeRequestId = null;
            }
            cancelButton.disabled = true;
            renderRunProgress({
                phase: "error",
                current: 0,
                total: 1,
                message: `${message.error.code}: ${message.error.message}`,
            });
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
    state.latestAuthIssue = null;
    updateRepoLoadControls();
    renderAuthState();
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
        const nextArgs = argsWithInputs(result.inputs);

        state.latestSource = result.source;
        state.latestIngest = result.ingest;
        state.latestLoadError = null;
        state.latestAuthIssue = null;
        renderRepoCapabilities();
        renderAuthState();
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
        const repoError = error instanceof Error ? error : new Error(String(error));
        if (repoError.ingest) {
            state.latestSource = {
                repo,
                ref,
                strategy: "github-tree-contents",
            };
            state.latestIngest = repoError.ingest;
        }
        state.latestLoadError = repoError;
        state.latestAuthIssue =
            authModeForToken(tokenInput.value) === "authenticated" &&
            isAuthRepairLoadError(repoError)
                ? {
                      code: repoError.code,
                  }
                : null;
        renderRepoCapabilities();
        renderAuthState();
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

loadLocalButton.addEventListener("click", async () => {
    const fileEntries = selectedLocalFileEntries();

    if (fileEntries.length === 0) {
        setStatus(loadStatusOutput, "choose local files or a directory first", "warning");
        return;
    }

    state.localLoadActive = true;
    state.latestLoadError = null;
    updateRepoLoadControls();
    renderLoadProgress({
        phase: "start",
        current: 0,
        total: fileEntries.length,
        message: `Reading ${fileEntries.length} local file(s)`,
    });
    setStatus(loadStatusOutput, `reading ${fileEntries.length} local file(s)...`, "working");

    try {
        const inputs = [];
        let bytesRead = 0;

        for (const [index, { file, path }] of fileEntries.entries()) {
            renderLoadProgress({
                phase: "files",
                current: index + 1,
                total: fileEntries.length,
                loadedFiles: inputs.length,
                message: `Reading ${path}`,
            });

            const text = await file.text();
            bytesRead += Number.isFinite(file?.size)
                ? file.size
                : new TextEncoder().encode(text).byteLength;
            inputs.push({ path, text });
        }

        if (inputs.length === 0) {
            throw new Error("No local files were loaded.");
        }

        const source = {
            repo: "local files",
            ref: "browser",
            strategy: "local-file-input",
        };
        const ingest = localIngestReceipt(fileEntries, inputs, bytesRead);

        state.latestSource = source;
        state.latestIngest = ingest;
        state.latestLoadError = null;
        renderRepoCapabilities();
        renderIngestSummary();
        argsInput.value = JSON.stringify(argsWithInputs(inputs), null, 2);
        appendLog("local files -> main", {
            source,
            ingest,
            samplePaths: inputs.slice(0, 5).map((input) => input.path),
        });
        renderLoadProgress({
            phase: "complete",
            current: inputs.length,
            total: inputs.length,
            loadedFiles: inputs.length,
            message: `Loaded ${inputs.length} local file(s)`,
        });
        setStatus(loadStatusOutput, `loaded ${inputs.length} local file(s)`, "success");
    } catch (error) {
        const localError = error instanceof Error ? error : new Error(String(error));
        state.latestLoadError = localError;
        renderRepoCapabilities();
        renderIngestSummary();
        appendLog("local files error -> main", sanitizeErrorForLog(localError));
        renderLoadProgress({
            phase: "error",
            current: 0,
            total: fileEntries.length,
            message: localError.message,
        });
        setStatus(loadStatusOutput, `local file load failed: ${localError.message}`, "error");
    } finally {
        state.localLoadActive = false;
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

retryLoadButton.addEventListener("click", () => {
    if (!isRetryableLoadError(state.latestLoadError)) {
        setStatus(loadStatusOutput, "no retryable repo load error", "warning");
        return;
    }

    loadRepoButton.click();
});

tokenInput.addEventListener("input", () => {
    state.latestAuthIssue = null;
    syncTokenState();
});

clearTokenButton.addEventListener("click", () => {
    tokenInput.value = "";
    state.latestAuthIssue = null;
    clearSessionToken(resolveSessionStorage());
    syncTokenState({ persist: false });
    setStatus(loadStatusOutput, "GitHub token cleared", "neutral");
});

window.addEventListener("beforeunload", () => {
    clearDownloadUrl();
});

modeInput.addEventListener("change", () => {
    setSampleArgs(modeInput.value);
});

runButton.addEventListener("click", () => {
    if (!selectableModes().includes(modeInput.value)) {
        setStatus(runStatusOutput, `${modeInput.value} is not available in this wasm bundle`, "error");
        return;
    }

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
    renderRunProgress({
        phase: "start",
        current: 0,
        total: 1,
        message: `sent ${requestId}`,
    });
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
syncTokenState({ persist: false });
renderRepoCapabilities();
renderIngestSummary();
updateRepoLoadControls();
setSampleArgs(modeInput.value);
updateRunControls();
