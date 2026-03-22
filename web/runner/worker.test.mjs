import test from "node:test";
import assert from "node:assert/strict";
import { Worker } from "node:worker_threads";

import { MESSAGE_TYPES } from "./messages.js";

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
