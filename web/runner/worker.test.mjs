import test from "node:test";
import assert from "node:assert/strict";
import { existsSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { Worker } from "node:worker_threads";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

import { MESSAGE_TYPES } from "./messages.js";

const HAS_REAL_WASM_BUNDLE =
    existsSync(new URL("./vendor/tokmd-wasm/tokmd_wasm.js", import.meta.url)) &&
    existsSync(new URL("./vendor/tokmd-wasm/tokmd_wasm_bg.wasm", import.meta.url));

function onceMessage(worker) {
    return new Promise((resolve, reject) => {
        const onMessage = (message) => {
            cleanup();
            resolve(message);
        };
        const onError = (error) => {
            cleanup();
            reject(error);
        };
        const cleanup = () => {
            worker.off("message", onMessage);
            worker.off("error", onError);
        };

        worker.on("message", onMessage);
        worker.on("error", onError);
    });
}

async function nextMessageOfType(worker, type) {
    const messages = [];

    while (true) {
        const message = await onceMessage(worker);
        messages.push(message);

        if (message.type === type) {
            return { message, messages };
        }
    }
}

function createMockWasmBundle(options = {}) {
    const tempDir = mkdtempSync(join(tmpdir(), "tokmd-mock-wasm-"));
    const wasmModulePath = join(tempDir, "tokmd_wasm.js");
    const wasmBinaryPath = join(tempDir, "tokmd_wasm_bg.wasm");

    const {
        includeDefault = true,
        includeVersion = true,
        includeSchemaVersion = true,
        includeRunLang = false,
        includeRunModule = false,
        includeRunExport = false,
        includeRunAnalyze = false,
        capabilities = null,
        capabilitiesRequiresInit = false,
        version = "1.9.0",
        schemaVersion = 2,
        analysisSchemaVersion = 9,
    } = options;

    const moduleSourceLines = [];
    if (capabilitiesRequiresInit) {
        moduleSourceLines.push("let initialized = false;");
    }

    if (includeDefault) {
        moduleSourceLines.push(
            capabilitiesRequiresInit
                ? "export default async function init() { initialized = true; }"
                : "export default async function init() {}"
        );
    }

    if (includeVersion) {
        moduleSourceLines.push(`export function version() { return "${version}"; }`);
    }

    if (includeSchemaVersion) {
        moduleSourceLines.push(`export function schemaVersion() { return ${schemaVersion}; }`);
    }

    if (includeRunLang) {
        moduleSourceLines.push(
            "export function runLang(args) { return { mode: 'lang', scan: { paths: args.inputs.map((input) => input.path) }, total: { files: args.inputs.length } }; }"
        );
    }

    if (includeRunModule) {
        moduleSourceLines.push(
            "export function runModule(args) { return { mode: 'module', rows: args.inputs.map((input) => ({ module: input.path, path: input.path })) }; }"
        );
    }

    if (includeRunExport) {
        moduleSourceLines.push(
            "export function runExport(args) { return { mode: 'export', rows: args.inputs.map((input) => ({ path: input.path })) }; }"
        );
    }

    if (includeRunAnalyze) {
        moduleSourceLines.push(
            `export function analysisSchemaVersion() { return ${analysisSchemaVersion}; }`,
            "export function runAnalyze(args) { return { mode: 'analysis', preset: args.preset ?? 'receipt', source: { inputs: args.inputs.map((input) => input.path) } }; }"
        );
    }

    if (capabilities !== null) {
        moduleSourceLines.push(
            capabilitiesRequiresInit
                ? `export function capabilities() { if (!initialized) { throw new Error("capabilities before init"); } return ${JSON.stringify(capabilities)}; }`
                : `export function capabilities() { return ${JSON.stringify(capabilities)}; }`
        );
    }

    writeFileSync(wasmModulePath, `${moduleSourceLines.join("\n")}\n`);
    writeFileSync(wasmBinaryPath, Buffer.from([0, 97, 115, 109]));

    return {
        cleanup() {
            rmSync(tempDir, { recursive: true, force: true });
        },
        moduleUrl: pathToFileURL(wasmModulePath).href,
        wasmBinaryPath,
    };
}

function createWorkerForMockWasm(options, workerData = {}) {
    const bundle = createMockWasmBundle(options);
    const worker = new Worker(new URL("./worker.js", import.meta.url), {
        type: "module",
        workerData: {
            wasmModuleUrl: bundle.moduleUrl,
            wasmBinaryPath: bundle.wasmBinaryPath,
            ...workerData,
        },
    });

    return {
        worker,
        cleanup: bundle.cleanup,
    };
}

test("worker publishes ready on boot", async () => {
    const worker = new Worker(new URL("./worker.js", import.meta.url), {
        type: "module",
        workerData: {
            runnerMode: "stub",
        },
    });

    try {
        const message = await onceMessage(worker);

        assert.equal(message.type, MESSAGE_TYPES.READY);
        assert.equal(message.capabilities.cancel, false);
        assert.equal(message.capabilities.downloads, true);
        assert.equal(message.capabilities.progress, true);
        assert.equal(message.capabilities.wasm, true);
        assert.equal(message.engine.version, "stub");
        assert.deepEqual(message.capabilities.modes, ["lang", "module", "export", "analyze"]);
        assert.deepEqual(message.capabilities.analyzePresets, ["receipt", "estimate"]);
    } finally {
        await worker.terminate();
    }
});

test("worker forwards nested scan inputs through the stub runner", async () => {
    const worker = new Worker(new URL("./worker.js", import.meta.url), {
        type: "module",
        workerData: {
            runnerMode: "stub",
        },
    });

    try {
        await onceMessage(worker);

        worker.postMessage({
            type: "run",
            requestId: "stub-scan-inputs",
            mode: "analyze",
            args: {
                scan: {
                    inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
                },
                preset: "estimate",
            },
        });

        const { message: result, messages } = await nextMessageOfType(
            worker,
            MESSAGE_TYPES.RESULT
        );
        assert.equal(result.type, MESSAGE_TYPES.RESULT);
        assert.equal(result.requestId, "stub-scan-inputs");
        assert.equal(result.data.mode, "analysis");
        assert.deepEqual(result.data.source.inputs, ["src/lib.rs"]);
        assert.deepEqual(
            messages.map((message) => message.type),
            [
                MESSAGE_TYPES.PROGRESS,
                MESSAGE_TYPES.PROGRESS,
                MESSAGE_TYPES.PROGRESS,
                MESSAGE_TYPES.PROGRESS,
                MESSAGE_TYPES.RESULT,
            ]
        );
        assert.deepEqual(
            messages
                .filter((message) => message.type === MESSAGE_TYPES.PROGRESS)
                .map((message) => message.phase),
            ["start", "fetch", "analyze", "done"]
        );
    } finally {
        await worker.terminate();
    }
});

test("worker advertises only supported modes from a minimal wasm module", async () => {
    const { worker, cleanup } = createWorkerForMockWasm({
        includeRunLang: true,
    });

    try {
        const message = await onceMessage(worker);

        assert.equal(message.type, MESSAGE_TYPES.READY);
        assert.deepEqual(message.capabilities.modes, ["lang"]);
        assert.deepEqual(message.capabilities.analyzePresets, []);
        assert.equal(message.engine.version, "1.9.0");

        worker.postMessage({
            type: "run",
            requestId: "mock-minimal",
            mode: "lang",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}" }],
            },
        });

        const { message: result } = await nextMessageOfType(worker, MESSAGE_TYPES.RESULT);
        assert.equal(result.type, MESSAGE_TYPES.RESULT);
        assert.equal(result.requestId, "mock-minimal");
        assert.equal(result.data.mode, "lang");
    } finally {
        await worker.terminate();
        cleanup();
    }
});

test("worker fails bootstrap when required exports are missing", async () => {
    for (const scenario of [
        {
            name: "version",
            options: {
                includeVersion: false,
                includeRunLang: true,
            },
        },
        {
            name: "schemaVersion",
            options: {
                includeSchemaVersion: false,
                includeRunLang: true,
            },
        },
        {
            name: "default",
            options: {
                includeDefault: false,
                includeRunLang: true,
            },
        },
    ]) {
        const { worker, cleanup } = createWorkerForMockWasm(scenario.options);

        try {
            const message = await onceMessage(worker);
            assert.equal(message.type, MESSAGE_TYPES.ERROR);
            assert.equal(message.error.code, "wasm_boot_failed");
            assert.match(message.error.message, new RegExp(`missing required exports: .*${scenario.name}`));
        } finally {
            await worker.terminate();
            cleanup();
        }
    }
});

test("worker fails bootstrap when no supported run mode is exported", async () => {
    const { worker, cleanup } = createWorkerForMockWasm({
        includeDefault: true,
        includeVersion: true,
        includeSchemaVersion: true,
    });

    try {
        const message = await onceMessage(worker);

        assert.equal(message.type, MESSAGE_TYPES.ERROR);
        assert.equal(message.error.code, "wasm_boot_failed");
        assert.match(message.error.message, /no supported run mode/i);
    } finally {
        await worker.terminate();
        cleanup();
    }
});

test("worker advertises analyze support when runAnalyze exists", async () => {
    const { worker, cleanup } = createWorkerForMockWasm({
        includeRunLang: true,
        includeRunAnalyze: true,
    });

    try {
        const message = await onceMessage(worker);

        assert.equal(message.type, MESSAGE_TYPES.READY);
        assert.deepEqual(message.capabilities.modes, ["lang", "analyze"]);
        assert.deepEqual(message.capabilities.analyzePresets, ["receipt", "estimate"]);
    } finally {
        await worker.terminate();
        cleanup();
    }
});

test("worker uses wasm capabilities payload for advertised modes and presets", async () => {
    const { worker, cleanup } = createWorkerForMockWasm({
        includeRunLang: true,
        includeRunModule: true,
        includeRunAnalyze: true,
        capabilities: {
            modes: ["lang", "analyze"],
            analyze: {
                rootlessPresets: ["estimate"],
            },
        },
    });

    try {
        const message = await onceMessage(worker);

        assert.equal(message.type, MESSAGE_TYPES.READY);
        assert.deepEqual(message.capabilities.modes, ["lang", "analyze"]);
        assert.deepEqual(message.capabilities.analyzePresets, ["estimate"]);

        worker.postMessage({
            type: "run",
            requestId: "mock-hidden-module",
            mode: "module",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}" }],
            },
        });

        const hiddenMode = await onceMessage(worker);
        assert.equal(hiddenMode.type, MESSAGE_TYPES.ERROR);
        assert.equal(hiddenMode.error.code, "unsupported_mode");

        worker.postMessage({
            type: "run",
            requestId: "mock-hidden-preset",
            mode: "analyze",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}" }],
                preset: "receipt",
            },
        });

        const hiddenPreset = await onceMessage(worker);
        assert.equal(hiddenPreset.type, MESSAGE_TYPES.ERROR);
        assert.equal(hiddenPreset.error.code, "unsupported_preset");

        worker.postMessage({
            type: "run",
            requestId: "mock-cap-estimate",
            mode: "analyze",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}" }],
                preset: "estimate",
            },
        });

        const { message: result } = await nextMessageOfType(worker, MESSAGE_TYPES.RESULT);
        assert.equal(result.type, MESSAGE_TYPES.RESULT);
        assert.equal(result.requestId, "mock-cap-estimate");
        assert.equal(result.data.mode, "analysis");
        assert.equal(result.data.preset, "estimate");
    } finally {
        await worker.terminate();
        cleanup();
    }
});

test("worker reads wasm capabilities payload only after initialization", async () => {
    const { worker, cleanup } = createWorkerForMockWasm({
        includeRunLang: true,
        capabilitiesRequiresInit: true,
        capabilities: {
            modes: ["lang"],
            analyze: {
                rootlessPresets: [],
            },
        },
    });

    try {
        const message = await onceMessage(worker);

        assert.equal(message.type, MESSAGE_TYPES.READY);
        assert.deepEqual(message.capabilities.modes, ["lang"]);
        assert.deepEqual(message.capabilities.analyzePresets, []);
    } finally {
        await worker.terminate();
        cleanup();
    }
});

test("worker does not advertise wasm-declared modes without matching exports", async () => {
    const { worker, cleanup } = createWorkerForMockWasm({
        includeRunLang: true,
        includeRunAnalyze: false,
        capabilities: {
            modes: ["lang", "analyze"],
            analyze: {
                rootlessPresets: ["receipt", "estimate"],
            },
        },
    });

    try {
        const message = await onceMessage(worker);

        assert.equal(message.type, MESSAGE_TYPES.READY);
        assert.deepEqual(message.capabilities.modes, ["lang"]);
        assert.deepEqual(message.capabilities.analyzePresets, []);
    } finally {
        await worker.terminate();
        cleanup();
    }
});

test("worker rejects analyze mode when runAnalyze is not exported", async () => {
    const { worker, cleanup } = createWorkerForMockWasm({
        includeRunLang: true,
        includeRunAnalyze: false,
    });

    try {
        await onceMessage(worker);

        worker.postMessage({
            type: "run",
            requestId: "mock-analyze",
            mode: "analyze",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}" }],
                preset: "receipt",
            },
        });

        const message = await onceMessage(worker);

        assert.equal(message.type, MESSAGE_TYPES.ERROR);
        assert.equal(message.error.code, "unsupported_mode");
    } finally {
        await worker.terminate();
        cleanup();
    }
});

test("worker forwards run messages through the runtime", async () => {
    const worker = new Worker(new URL("./worker.js", import.meta.url), {
        type: "module",
        workerData: {
            runnerMode: "stub",
        },
    });

    try {
        await onceMessage(worker);

        worker.postMessage({
            type: "run",
            requestId: "run-3",
            mode: "analyze",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
            },
        });

        const { message } = await nextMessageOfType(worker, MESSAGE_TYPES.RESULT);

        assert.equal(message.type, MESSAGE_TYPES.RESULT);
        assert.equal(message.requestId, "run-3");
        assert.equal(message.data.mode, "analysis");
        assert.equal(message.data.preset, "receipt");
    } finally {
        await worker.terminate();
    }
});

test(
    "worker boots the real tokmd-wasm bundle when it has been built",
    async (t) => {
        if (!HAS_REAL_WASM_BUNDLE) {
            t.skip("built tokmd-wasm bundle not present");
            return;
        }

        const worker = new Worker(new URL("./worker.js", import.meta.url), {
            type: "module",
        });

        try {
            const ready = await onceMessage(worker);

            assert.equal(ready.type, MESSAGE_TYPES.READY);
            assert.equal(ready.capabilities.wasm, true);
            assert.notEqual(ready.engine.version, "stub");
            assert.ok(ready.engine.schemaVersion > 0);

            worker.postMessage({
                type: "run",
                requestId: "run-real-lang",
                mode: "lang",
                args: {
                    inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
                    files: true,
                },
            });

            const { message: result } = await nextMessageOfType(
                worker,
                MESSAGE_TYPES.RESULT
            );

            assert.equal(result.type, MESSAGE_TYPES.RESULT);
            assert.equal(result.requestId, "run-real-lang");
            assert.equal(result.data.mode, "lang");
            assert.equal(result.data.total.files, 1);
            assert.equal(result.data.scan.paths[0], "src/lib.rs");

            worker.postMessage({
                type: "run",
                requestId: "run-real-estimate",
                mode: "analyze",
                args: {
                    inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
                    preset: "estimate",
                },
            });

            const { message: analyze } = await nextMessageOfType(
                worker,
                MESSAGE_TYPES.RESULT
            );

            assert.equal(analyze.type, MESSAGE_TYPES.RESULT);
            assert.equal(analyze.requestId, "run-real-estimate");
            assert.equal(analyze.data.mode, "analysis");
            assert.equal(analyze.data.args.preset, "estimate");
            assert.equal(analyze.data.source.inputs[0], "src/lib.rs");
            assert.equal(analyze.data.effort.model, "cocomo81-basic");
        } finally {
            await worker.terminate();
        }
    }
);
