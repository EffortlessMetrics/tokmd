import test from "node:test";
import assert from "node:assert/strict";

import { createCancelMessage, createRunMessage, MESSAGE_TYPES } from "./messages.js";
import { handleRunnerMessage, isProtocolMessage } from "./runtime.js";

function createStubRunner() {
    return {
        runLang(args) {
            return {
                mode: "lang",
                total: { files: args.inputs.length },
            };
        },
        runModule() {
            return { mode: "module" };
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
                source: {
                    inputs: args.inputs.map((input) => input.path),
                },
                preset: args.preset ?? "receipt",
            };
        },
    };
}

test("runtime rejects malformed messages", async () => {
    const message = await handleRunnerMessage({ type: "bogus" });

    assert.equal(message.type, MESSAGE_TYPES.ERROR);
    assert.equal(message.error.code, "invalid_message");
    assert.equal(message.requestId, null);
    assert.equal(isProtocolMessage(message), true);
});

test("runtime rejects run messages without valid inputs", async () => {
    const message = await handleRunnerMessage(
        createRunMessage({
            requestId: "run-2",
            mode: "lang",
            args: {},
        })
    );

    assert.equal(message.type, MESSAGE_TYPES.ERROR);
    assert.equal(message.error.code, "invalid_message");
});

test("runtime reserves cancel without promising it", async () => {
    const message = await handleRunnerMessage(createCancelMessage("run-7"));

    assert.equal(message.type, MESSAGE_TYPES.ERROR);
    assert.equal(message.requestId, "run-7");
    assert.equal(message.error.code, "cancel_unavailable");
});

test("runtime returns results once a runner is available", async () => {
    const message = await handleRunnerMessage(
        createRunMessage({
            requestId: "run-8",
            mode: "lang",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
            },
        }),
        { runner: createStubRunner() }
    );

    assert.equal(message.type, MESSAGE_TYPES.RESULT);
    assert.equal(message.requestId, "run-8");
    assert.equal(message.data.mode, "lang");
    assert.equal(message.data.total.files, 1);
});

test("analyze without preset defaults to receipt and returns a result", async () => {
    const message = await handleRunnerMessage(
        createRunMessage({
            requestId: "run-9",
            mode: "analyze",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
            },
        }),
        { runner: createStubRunner() }
    );

    assert.equal(message.type, MESSAGE_TYPES.RESULT);
    assert.equal(message.requestId, "run-9");
    assert.equal(message.data.mode, "analysis");
    assert.equal(message.data.preset, "receipt");
});

test("analyze rejects unsupported presets before runner execution", async () => {
    const message = await handleRunnerMessage(
        createRunMessage({
            requestId: "run-10",
            mode: "analyze",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
                preset: "health",
            },
        }),
        { runner: createStubRunner() }
    );

    assert.equal(message.type, MESSAGE_TYPES.ERROR);
    assert.equal(message.error.code, "unsupported_preset");
});

test("runtime reports boot failures against run requests", async () => {
    const message = await handleRunnerMessage(
        createRunMessage({
            requestId: "run-11",
            mode: "export",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
            },
        }),
        { bootError: new Error("missing tokmd_wasm.js") }
    );

    assert.equal(message.type, MESSAGE_TYPES.ERROR);
    assert.equal(message.error.code, "wasm_boot_failed");
    assert.match(message.error.message, /missing tokmd_wasm\.js/);
});
