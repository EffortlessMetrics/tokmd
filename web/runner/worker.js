import { createErrorMessage, createReadyMessage, normalizeAnalyzePreset } from "./messages.js";
import { handleRunnerMessage } from "./runtime.js";

let emitMessage;
let subscribe;
let nodeWorkerData = null;
let isNodeWorker = false;

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
    };
}

async function loadTokmdRunner() {
    if (nodeWorkerData?.runnerMode === "stub") {
        return createStubRunner();
    }

    const moduleUrl = new URL("./vendor/tokmd-wasm/tokmd_wasm.js", import.meta.url);
    const wasmModule = await import(moduleUrl.href);
    if (isNodeWorker) {
        const { readFile } = await import("node:fs/promises");
        const wasmUrl = new URL("./vendor/tokmd-wasm/tokmd_wasm_bg.wasm", import.meta.url);
        await wasmModule.default({ module_or_path: await readFile(wasmUrl) });
    } else {
        await wasmModule.default();
    }

    return {
        runLang(args) {
            return wasmModule.runLang(args);
        },
        runModule(args) {
            return wasmModule.runModule(args);
        },
        runExport(args) {
            return wasmModule.runExport(args);
        },
        runAnalyze(args) {
            return wasmModule.runAnalyze(args);
        },
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
        emitMessage(await handleRunnerMessage(message, { runner, bootError }));
    });
});
