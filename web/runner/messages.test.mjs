import test from "node:test";
import assert from "node:assert/strict";

import {
    MESSAGE_TYPES,
    RUNNER_PROTOCOL_VERSION,
    SUPPORTED_ANALYZE_PRESETS,
    SUPPORTED_MODES,
    createCancelMessage,
    createReadyMessage,
    createRunMessage,
    isCancelMessage,
    isInMemoryInput,
    isRunMessage,
    normalizeAnalyzePreset,
} from "./messages.js";

test("ready message exposes protocol version and capabilities", () => {
    const message = createReadyMessage();

    assert.equal(message.type, MESSAGE_TYPES.READY);
    assert.equal(message.protocolVersion, RUNNER_PROTOCOL_VERSION);
    assert.deepEqual(message.capabilities.modes, [...SUPPORTED_MODES]);
    assert.deepEqual(
        message.capabilities.analyzePresets,
        [...SUPPORTED_ANALYZE_PRESETS]
    );
    assert.equal(message.capabilities.wasm, false);
    assert.equal(message.capabilities.zipball, false);
});

test("normalizeAnalyzePreset defaults to receipt", () => {
    assert.equal(normalizeAnalyzePreset({}), "receipt");
    assert.equal(normalizeAnalyzePreset({ preset: "Estimate" }), "estimate");
    assert.equal(
        normalizeAnalyzePreset({ analyze: { preset: "Receipt" } }),
        "receipt"
    );
});

test("run and cancel helpers produce valid protocol messages", () => {
    const run = createRunMessage({
        requestId: "run-1",
        mode: "lang",
        args: { inputs: [] },
    });
    const cancel = createCancelMessage("run-1");

    assert.equal(isRunMessage(run), true);
    assert.equal(isCancelMessage(cancel), true);
    assert.equal(isRunMessage(cancel), false);
});

test("run messages require valid inputs, paths, or scan object", () => {
    assert.equal(
        isInMemoryInput({ path: "src/lib.rs", text: "pub fn alpha() {}\n" }),
        true
    );
    assert.equal(
        isInMemoryInput({
            path: "src/lib.rs",
            base64: "cHViIGZuIGFscGhhKCkge30K",
        }),
        true
    );
    assert.equal(
        isRunMessage({
            type: "run",
            requestId: "b64",
            mode: "lang",
            args: {
                inputs: [{ path: "src/lib.rs", base64: "cHViIGZuIGFscGhhKCkge30K" }],
            },
        }),
        true
    );

    assert.equal(
        isRunMessage({
            type: "run",
            requestId: "paths-1",
            mode: "lang",
            args: {
                paths: ["src/lib.rs"],
            },
        }),
        true
    );

    assert.equal(
        isRunMessage({
            type: "run",
            requestId: "scan-1",
            mode: "lang",
            args: {
                scan: { path: "src" },
            },
        }),
        true
    );

    assert.equal(
        isInMemoryInput({
            path: "src/lib.rs",
            text: "pub fn alpha() {}\n",
            base64: "cHViIGZuIGFscGhhKCkge30K",
        }),
        false
    );
    assert.equal(isRunMessage({ type: "run", requestId: "x", mode: "lang", args: {} }), false);
    assert.equal(
        isRunMessage({
            type: "run",
            requestId: "x",
            mode: "lang",
            args: { inputs: [{ path: "", text: "bad\n" }] },
        }),
        false
    );
    assert.equal(
        isRunMessage({
            type: "run",
            requestId: "x",
            mode: "lang",
            args: { paths: [123] },
        }),
        false
    );
});
