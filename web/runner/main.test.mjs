import assert from "node:assert/strict";
import test from "node:test";

function createMemoryStorage(initial = {}) {
    const values = new Map(Object.entries(initial));

    return {
        getItem(key) {
            return values.has(key) ? values.get(key) : null;
        },
        removeItem(key) {
            values.delete(key);
        },
        setItem(key, value) {
            values.set(key, String(value));
        },
    };
}

class FakeElement {
    constructor(tagName = "div", value = "") {
        this.tagName = tagName.toUpperCase();
        this.children = [];
        this.className = "";
        this.dataset = {};
        this.disabled = false;
        this.hidden = false;
        this.max = 1;
        this.options = [];
        this.textContent = "";
        this.value = value;
        this.listeners = new Map();
    }

    addEventListener(type, handler) {
        const listeners = this.listeners.get(type) ?? [];
        listeners.push(handler);
        this.listeners.set(type, listeners);
    }

    append(...children) {
        this.children.push(...children);
    }

    prepend(...children) {
        this.children.unshift(...children);
    }

    replaceChildren(...children) {
        this.children = [...children];
    }

    removeAttribute(name) {
        if (name === "value") {
            this.value = "";
        }
    }

    click() {
        const listeners = this.listeners.get("click") ?? [];
        this.lastClickPromise = Promise.all(
            listeners.map((listener) => listener({ target: this }))
        );
        return this.lastClickPromise;
    }

    dispatchEvent(event) {
        const type = typeof event === "string" ? event : event.type;
        const listeners = this.listeners.get(type) ?? [];
        this.lastEventPromise = Promise.all(
            listeners.map((listener) => listener({ target: this, type }))
        );
        return this.lastEventPromise;
    }
}

class FakeWorker {
    static instances = [];

    constructor(url, options = {}) {
        this.url = url;
        this.options = options;
        this.listeners = new Map();
        this.messages = [];
        FakeWorker.instances.push(this);
    }

    addEventListener(type, handler) {
        const listeners = this.listeners.get(type) ?? [];
        listeners.push(handler);
        this.listeners.set(type, listeners);
    }

    postMessage(message) {
        this.messages.push(message);
    }

    emit(message) {
        for (const listener of this.listeners.get("message") ?? []) {
            listener({ data: message });
        }
    }
}

function createDocumentHarness() {
    const defaults = new Map([
        ["[data-repo]", "EffortlessMetrics/tokmd"],
        ["[data-ref]", "main"],
        ["[data-token]", ""],
        ["[data-auth-state]", ""],
        ["[data-mode]", "lang"],
        ["[data-args]", ""],
        ["[data-load-repo]", ""],
        ["[data-retry-load]", ""],
        ["[data-cancel-load]", ""],
        ["[data-run]", ""],
        ["[data-cancel]", ""],
        ["[data-download]", ""],
        ["[data-load-status]", ""],
        ["[data-run-status]", ""],
        ["[data-worker-capabilities]", ""],
        ["[data-repo-capabilities]", ""],
        ["[data-ingest-summary]", ""],
        ["[data-load-progress-panel]", ""],
        ["[data-load-progress]", ""],
        ["[data-load-progress-text]", ""],
        ["[data-run-progress-panel]", ""],
        ["[data-run-progress]", ""],
        ["[data-run-progress-text]", ""],
        ["[data-result]", "waiting for first result..."],
        ["[data-log]", ""],
        ["[data-clear-token]", ""],
    ]);
    const elements = new Map(
        [...defaults].map(([selector, value]) => [selector, new FakeElement("div", value)])
    );

    elements.get("[data-load-progress-panel]").hidden = true;
    elements.get("[data-run-progress-panel]").hidden = true;
    elements.get("[data-retry-load]").disabled = true;
    elements.get("[data-cancel-load]").disabled = true;
    elements.get("[data-cancel]").disabled = true;
    elements.get("[data-download]").disabled = true;

    const modeInput = elements.get("[data-mode]");
    modeInput.tagName = "SELECT";
    modeInput.options = ["lang", "module", "export", "analyze"].map((value) => {
        const option = new FakeElement("option", value);
        option.value = value;
        return option;
    });

    return {
        document: {
            querySelector(selector) {
                const element = elements.get(selector);
                if (!element) {
                    throw new Error(`unexpected selector ${selector}`);
                }
                return element;
            },
            createElement(tagName) {
                return new FakeElement(tagName);
            },
        },
        element(selector) {
            return elements.get(selector);
        },
    };
}

function collectText(element) {
    return [
        element.textContent,
        ...element.children.map((child) => collectText(child)),
    ].join("\n");
}

function installBrowserHarness(t, { fetchImpl, storage }) {
    const harness = createDocumentHarness();
    const originalDocument = globalThis.document;
    const originalWindow = globalThis.window;
    const originalWorker = globalThis.Worker;
    const originalFetch = globalThis.fetch;
    const originalSessionStorage = globalThis.sessionStorage;
    const originalCreateObjectUrl = URL.createObjectURL;
    const originalRevokeObjectUrl = URL.revokeObjectURL;

    FakeWorker.instances = [];
    globalThis.document = harness.document;
    globalThis.window = {
        addEventListener() {},
    };
    globalThis.Worker = FakeWorker;
    globalThis.fetch = fetchImpl;
    globalThis.sessionStorage = storage;
    URL.createObjectURL = () => "blob:tokmd-browser-runner-test";
    URL.revokeObjectURL = () => {};

    t.after(() => {
        globalThis.document = originalDocument;
        globalThis.window = originalWindow;
        globalThis.Worker = originalWorker;
        globalThis.fetch = originalFetch;
        globalThis.sessionStorage = originalSessionStorage;
        URL.createObjectURL = originalCreateObjectUrl;
        URL.revokeObjectURL = originalRevokeObjectUrl;
    });

    return harness;
}

function jsonResponse(value, init = {}) {
    return new Response(JSON.stringify(value), {
        status: init.status ?? 200,
        headers: {
            "content-type": "application/json",
            ...(init.headers ?? {}),
        },
    });
}

function textResponse(value) {
    return new Response(value, {
        status: 200,
        headers: {
            "content-type": "text/plain; charset=utf-8",
        },
    });
}

test("main page wires token state, retryable repo loads, cache display, and result preservation", async (t) => {
    const fetchCalls = [];
    let treeAttempts = 0;
    const storage = createMemoryStorage({
        "tokmd.githubToken": "  ghp_saved  ",
    });
    const harness = installBrowserHarness(t, {
        storage,
        fetchImpl: async (url, options = {}) => {
            fetchCalls.push({
                url,
                authorization: options.headers?.Authorization ?? null,
            });

            if (url.includes("/git/trees/")) {
                treeAttempts += 1;
                if (treeAttempts === 1) {
                    return jsonResponse(
                        { message: "You have exceeded a secondary rate limit." },
                        {
                            status: 429,
                            headers: {
                                "retry-after": "12",
                            },
                        }
                    );
                }

                return jsonResponse({
                    tree: [{ path: "README.md", size: 32, type: "blob" }],
                });
            }

            if (url.includes("/contents/README.md")) {
                return textResponse("# tokmd\n");
            }

            throw new Error(`unexpected fetch url: ${url}`);
        },
    });

    await import(`./main.js?smoke=${Date.now()}`);
    const worker = FakeWorker.instances[0];

    worker.emit({
        type: "ready",
        protocolVersion: 2,
        capabilities: {
            modes: ["lang", "module", "export", "analyze"],
            analyzePresets: ["receipt", "estimate"],
            wasm: true,
            downloads: true,
            progress: true,
            cancel: false,
            zipball: false,
        },
        engine: {
            version: "test",
            schemaVersion: 2,
            analysisSchemaVersion: 9,
        },
    });

    const tokenInput = harness.element("[data-token]");
    const authState = harness.element("[data-auth-state]");
    const clearTokenButton = harness.element("[data-clear-token]");
    const runButton = harness.element("[data-run]");
    const loadRepoButton = harness.element("[data-load-repo]");
    const retryLoadButton = harness.element("[data-retry-load]");
    const resultOutput = harness.element("[data-result]");
    const repoCapabilitiesOutput = harness.element("[data-repo-capabilities]");
    const loadStatusOutput = harness.element("[data-load-status]");
    const runStatusOutput = harness.element("[data-run-status]");
    const loadProgressPanel = harness.element("[data-load-progress-panel]");
    const loadProgressText = harness.element("[data-load-progress-text]");
    const runProgressPanel = harness.element("[data-run-progress-panel]");
    const runProgressText = harness.element("[data-run-progress-text]");
    const logOutput = harness.element("[data-log]");

    assert.equal(tokenInput.value, "ghp_saved");
    assert.equal(authState.textContent, "authenticated");
    assert.equal(clearTokenButton.disabled, false);
    assert.match(harness.element("[data-worker-capabilities]").textContent, /downloads: yes/);

    await runButton.click();
    const runMessage = worker.messages.at(-1);
    assert.equal(runMessage.type, "run");
    assert.equal(runProgressPanel.hidden, false);
    assert.match(runProgressText.textContent, /sent run-1/);
    worker.emit({
        type: "progress",
        requestId: "stale-run",
        phase: "analyze",
        message: "stale worker progress",
    });
    assert.doesNotMatch(runProgressText.textContent, /stale/);
    worker.emit({
        type: "progress",
        requestId: runMessage.requestId,
        phase: "start",
        mode: "lang",
        message: "Starting lang run",
    });
    assert.match(runProgressText.textContent, /Starting lang run/);
    worker.emit({
        type: "result",
        requestId: runMessage.requestId,
        data: {
            mode: "lang",
            total: { files: 1 },
        },
    });
    const resultBeforeRepoError = resultOutput.textContent;
    assert.match(resultBeforeRepoError, /"mode": "lang"/);
    assert.match(runProgressText.textContent, /completed run-1/);

    await runButton.click();
    const secondRunMessage = worker.messages.at(-1);
    assert.equal(secondRunMessage.type, "run");
    assert.equal(secondRunMessage.requestId, "run-2");
    assert.match(runProgressText.textContent, /sent run-2/);

    worker.emit({
        type: "result",
        requestId: runMessage.requestId,
        data: {
            mode: "export",
            stale: true,
        },
    });
    assert.equal(resultOutput.textContent, resultBeforeRepoError);
    assert.match(runProgressText.textContent, /sent run-2/);
    assert.match(runStatusOutput.textContent, /sent run-2/);

    worker.emit({
        type: "error",
        requestId: runMessage.requestId,
        error: {
            code: "stale_worker_error",
            message: "stale worker terminal error",
        },
    });
    assert.equal(resultOutput.textContent, resultBeforeRepoError);
    assert.match(runProgressText.textContent, /sent run-2/);
    assert.doesNotMatch(runStatusOutput.textContent, /stale worker terminal error/);

    worker.emit({
        type: "result",
        requestId: secondRunMessage.requestId,
        data: {
            mode: "lang",
            total: { files: 2 },
        },
    });
    const secondResultBeforeRepoError = resultOutput.textContent;
    assert.match(secondResultBeforeRepoError, /"files": 2/);
    assert.match(runProgressText.textContent, /completed run-2/);

    await loadRepoButton.click();

    assert.equal(fetchCalls.length, 1);
    assert.equal(fetchCalls[0].authorization, "token ghp_saved");
    assert.equal(retryLoadButton.disabled, false);
    assert.equal(loadProgressPanel.hidden, false);
    assert.match(loadProgressText.textContent, /Retry after 12s/);
    assert.match(loadStatusOutput.textContent, /repo load failed:/);
    assert.equal(resultOutput.textContent, secondResultBeforeRepoError);
    assert.doesNotMatch(collectText(logOutput), /ghp_saved/);

    await retryLoadButton.click();
    await loadRepoButton.lastClickPromise;

    assert.equal(fetchCalls.length, 3);
    assert.equal(retryLoadButton.disabled, true);
    assert.match(loadStatusOutput.textContent, /loaded 1 file\(s\)/);
    assert.match(repoCapabilitiesOutput.textContent, /lastAuthMode: token/);
    assert.match(repoCapabilitiesOutput.textContent, /lastCache: memory miss/);
    assert.match(harness.element("[data-args]").value, /README\.md/);
    assert.equal(resultOutput.textContent, secondResultBeforeRepoError);

    await loadRepoButton.click();

    assert.equal(fetchCalls.length, 3);
    assert.match(repoCapabilitiesOutput.textContent, /lastCache: memory hit/);
    assert.equal(resultOutput.textContent, secondResultBeforeRepoError);

    await clearTokenButton.click();

    assert.equal(tokenInput.value, "");
    assert.equal(storage.getItem("tokmd.githubToken"), null);
    assert.equal(authState.textContent, "anonymous");
    assert.equal(clearTokenButton.disabled, true);
});

test("main page constrains mode controls to worker capabilities", async (t) => {
    const harness = installBrowserHarness(t, {
        storage: createMemoryStorage(),
        fetchImpl: async () => {
            throw new Error("fetch should not be called");
        },
    });

    await import(`./main.js?modeCapabilities=${Date.now()}`);
    const worker = FakeWorker.instances[0];
    const modeInput = harness.element("[data-mode]");
    const runButton = harness.element("[data-run]");
    const argsInput = harness.element("[data-args]");

    modeInput.value = "module";

    worker.emit({
        type: "ready",
        protocolVersion: 2,
        capabilities: {
            modes: ["lang", "analyze"],
            analyzePresets: ["receipt"],
            wasm: true,
            downloads: true,
            progress: true,
            cancel: false,
            zipball: false,
        },
        engine: {
            version: "test",
            schemaVersion: 2,
            analysisSchemaVersion: 9,
        },
    });

    assert.equal(modeInput.value, "lang");
    assert.equal(runButton.disabled, false);
    assert.equal(modeInput.options.find((option) => option.value === "lang").disabled, false);
    assert.equal(modeInput.options.find((option) => option.value === "module").disabled, true);
    assert.equal(modeInput.options.find((option) => option.value === "export").disabled, true);
    assert.equal(modeInput.options.find((option) => option.value === "analyze").disabled, false);

    modeInput.value = "analyze";
    await modeInput.dispatchEvent({ type: "change" });

    assert.match(argsInput.value, /"preset": "receipt"/);

    worker.emit({
        type: "ready",
        protocolVersion: 2,
        capabilities: {
            modes: ["analyze"],
            analyzePresets: [],
            wasm: true,
            downloads: true,
            progress: true,
            cancel: false,
            zipball: false,
        },
        engine: {
            version: "test",
            schemaVersion: 2,
            analysisSchemaVersion: 9,
        },
    });

    assert.equal(runButton.disabled, true);
    assert.equal(modeInput.options.find((option) => option.value === "analyze").disabled, true);
});
