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
