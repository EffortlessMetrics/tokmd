import {
    createErrorMessage,
    createReadyMessage,
    normalizeAnalyzePreset,
    SUPPORTED_ANALYZE_PRESETS,
    SUPPORTED_MODES,
} from "./messages.js";
import { handleRunnerMessage } from "./runtime.js";

let emitMessage;
let subscribe;
let nodeWorkerData = null;
let isNodeWorker = false;
const DEFAULT_WASM_MODULE_URL = new URL("./vendor/tokmd-wasm/tokmd_wasm.js", import.meta.url);
const DEFAULT_WASM_BINARY_URL = new URL("./vendor/tokmd-wasm/tokmd_wasm_bg.wasm", import.meta.url);

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
    const { parentPort, workerData } = await import("node:worker_threads");
    isNodeWorker = true;
    nodeWorkerData = workerData;

    emitMessage = (message) => parentPort.postMessage(message);
    subscribe = (handler) => {
        parentPort.on("message", handler);
    };
}

function createStubRunner() {
    const supportedModes = [...SUPPORTED_MODES];

    return {
        runLang(args) {
            return {
                mode: "lang",
                scan: {
                    paths: args.inputs.map((input) => input.path),
                },
                total: {
                    files: args.inputs.length,
                },
            };
        },
        runModule(args) {
            return {
                mode: "module",
                rows: args.inputs.map((input) => ({ module: input.path })),
            };
        },
        runExport(args) {
            return {
                mode: "export",
                rows: args.inputs.map((input) => ({ path: input.path })),
            };
        },
        runAnalyze(args) {
            return {
                mode: "analysis",
                preset: normalizeAnalyzePreset(args),
                source: {
                    inputs: args.inputs.map((input) => input.path),
                },
            };
        },
        engine: {
            version: "stub",
            schemaVersion: 0,
            analysisSchemaVersion: 0,
        },
        capabilities: {
            modes: supportedModes,
            analyzePresets: [...SUPPORTED_ANALYZE_PRESETS],
            missingExports: [],
        },
    };
}

function describeMissingExports(wasmModule) {
    const missing = [];

    const requiredFunctions = {
        default: "default",
        version: "version",
        schemaVersion: "schemaVersion",
    };

    for (const [key, symbol] of Object.entries(requiredFunctions)) {
        if (typeof wasmModule[key] !== "function") {
            missing.push(symbol);
        }
    }

    return missing;
}

function buildModeCapabilities(wasmModule) {
    const modes = [];
    if (typeof wasmModule.runLang === "function") {
        modes.push("lang");
    }

    if (typeof wasmModule.runModule === "function") {
        modes.push("module");
    }

    if (typeof wasmModule.runExport === "function") {
        modes.push("export");
    }

    if (typeof wasmModule.runAnalyze === "function") {
        modes.push("analyze");
    }

    return {
        modes,
        analyzePresets: typeof wasmModule.runAnalyze === "function" ? [...SUPPORTED_ANALYZE_PRESETS] : [],
    };
}

function createModeHandler(wasmModule, exportName, label) {
    if (typeof wasmModule[exportName] === "function") {
        return (args) => wasmModule[exportName](args);
    }

    return () => {
        throw new Error(`tokmd-wasm bundle does not provide ${label}`);
    };
}

function createMissingExportsError(missingExports) {
    const missing = missingExports.join(", ");

    return new Error(`tokmd-wasm bundle is missing required exports: ${missing}`);
}

function createRunnerFromWasmModule(wasmModule) {
    const missingExports = describeMissingExports(wasmModule);
    if (missingExports.length > 0) {
        throw createMissingExportsError(missingExports);
    }

    const capabilities = {
        ...buildModeCapabilities(wasmModule),
        missingExports,
    };

    return {
        runLang: createModeHandler(wasmModule, "runLang", "lang mode"),
        runModule: createModeHandler(wasmModule, "runModule", "module mode"),
        runExport: createModeHandler(wasmModule, "runExport", "export mode"),
        runAnalyze: createModeHandler(wasmModule, "runAnalyze", "analyze mode"),
        capabilities,
        engine: {
            version: wasmModule.version(),
            schemaVersion: wasmModule.schemaVersion(),
            analysisSchemaVersion:
                typeof wasmModule.analysisSchemaVersion === "function"
                    ? wasmModule.analysisSchemaVersion()
                    : null,
        },
    };
}

function resolveWasmModuleUrl() {
    if (typeof nodeWorkerData?.wasmModuleUrl === "string" && nodeWorkerData.wasmModuleUrl.trim()) {
        return nodeWorkerData.wasmModuleUrl;
    }

    return DEFAULT_WASM_MODULE_URL.href;
}

function resolveWasmBinaryPath() {
    if (typeof nodeWorkerData?.wasmBinaryPath === "string" && nodeWorkerData.wasmBinaryPath.trim()) {
        return nodeWorkerData.wasmBinaryPath;
    }

    return DEFAULT_WASM_BINARY_URL;
}

async function loadTokmdRunner() {
    if (nodeWorkerData?.runnerMode === "stub") {
        return createStubRunner();
    }

    const moduleUrl = resolveWasmModuleUrl();
    const wasmModule = await import(moduleUrl);
    const missingExports = describeMissingExports(wasmModule);
    if (missingExports.length > 0) {
        throw createMissingExportsError(missingExports);
    }

    const modeCapabilities = buildModeCapabilities(wasmModule);
    const hasAnyModes = modeCapabilities.modes.length > 0;

    if (!hasAnyModes) {
        throw new Error("tokmd-wasm bundle exposes no supported run modes");
    }

    if (isNodeWorker) {
        const { readFile } = await import("node:fs/promises");
        const wasmPath = resolveWasmBinaryPath();
        await wasmModule.default({ module_or_path: await readFile(wasmPath) });
    } else {
        await wasmModule.default();
    }

    return createRunnerFromWasmModule(wasmModule);
}

let runner = null;
let bootError = null;

const runnerReady = loadTokmdRunner()
    .then((loadedRunner) => {
        runner = loadedRunner;
        emitMessage(
            createReadyMessage({
                capabilities: {
                    wasm: true,
                    downloads: true,
                    modes: loadedRunner.capabilities.modes,
                    analyzePresets: loadedRunner.capabilities.analyzePresets,
                },
                engine: loadedRunner.engine,
            })
        );
        return loadedRunner;
    })
    .catch((error) => {
        bootError = error;
        emitMessage(
            createErrorMessage(
                null,
                "wasm_boot_failed",
                `browser runner failed to initialize tokmd-wasm: ${error instanceof Error ? error.message : String(error)}`
            )
        );
        return null;
    });

subscribe((message) => {
    void runnerReady.then(async () => {
        emitMessage(await handleRunnerMessage(message, {
            runner,
            runnerCapabilities: runner?.capabilities ?? {},
            bootError,
        }));
    });
});
