import test from "node:test";
import assert from "node:assert/strict";
import { existsSync } from "node:fs";
import { Worker } from "node:worker_threads";

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
        assert.equal(message.capabilities.wasm, true);
        assert.equal(message.engine.version, "stub");
    } finally {
        await worker.terminate();
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

        const message = await onceMessage(worker);

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

            const result = await onceMessage(worker);

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

            const analyze = await onceMessage(worker);

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
