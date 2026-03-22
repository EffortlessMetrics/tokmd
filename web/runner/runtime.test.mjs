import test from "node:test";
import assert from "node:assert/strict";

import { createCancelMessage, createRunMessage, MESSAGE_TYPES } from "./messages.js";
import { handleRunnerMessage, isProtocolMessage } from "./runtime.js";

test("runtime rejects malformed messages", () => {
    const message = handleRunnerMessage({ type: "bogus" });

    assert.equal(message.type, MESSAGE_TYPES.ERROR);
    assert.equal(message.error.code, "invalid_message");
    assert.equal(message.requestId, null);
    assert.equal(isProtocolMessage(message), true);
});

test("runtime reserves cancel without promising it", () => {
    const message = handleRunnerMessage(createCancelMessage("run-7"));

    assert.equal(message.type, MESSAGE_TYPES.ERROR);
    assert.equal(message.requestId, "run-7");
    assert.equal(message.error.code, "cancel_unavailable");
});

test("analyze without preset defaults to receipt and stays protocol-valid", () => {
    const message = handleRunnerMessage(
        createRunMessage({
            requestId: "run-9",
            mode: "analyze",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
            },
        })
    );

    assert.equal(message.type, MESSAGE_TYPES.ERROR);
    assert.equal(message.requestId, "run-9");
    assert.equal(message.error.code, "runner_not_wired");
});

test("analyze rejects unsupported presets before wasm wiring", () => {
    const message = handleRunnerMessage(
        createRunMessage({
            requestId: "run-10",
            mode: "analyze",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
                preset: "health",
            },
        })
    );

    assert.equal(message.type, MESSAGE_TYPES.ERROR);
    assert.equal(message.error.code, "unsupported_preset");
});
