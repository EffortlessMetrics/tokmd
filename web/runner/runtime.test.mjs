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

test("runtime rejects run messages without valid inputs and retains requestId", async () => {
    const message = await handleRunnerMessage(
        createRunMessage({
            requestId: "run-2",
            mode: "lang",
            args: {},
        })
    );

    assert.equal(message.type, MESSAGE_TYPES.ERROR);
    assert.equal(message.error.code, "invalid_message");
    assert.equal(message.requestId, "run-2");
});

test("runtime uses runner-provided mode capabilities", async () => {
    const message = await handleRunnerMessage(
        createRunMessage({
            requestId: "run-mode-cap",
            mode: "lang",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
            },
        }),
        {
            runner: createStubRunner(),
            runnerCapabilities: {
                modes: ["export"],
            },
        }
    );

    assert.equal(message.type, MESSAGE_TYPES.ERROR);
    assert.equal(message.error.code, "unsupported_mode");
    assert.match(message.error.message, /supports only export/);
});

test("runtime treats explicit empty mode capabilities as no supported modes", async () => {
    const message = await handleRunnerMessage(
        createRunMessage({
            requestId: "run-mode-empty-cap",
            mode: "lang",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
            },
        }),
        {
            runner: createStubRunner(),
            runnerCapabilities: {
                modes: [],
            },
        }
    );

    assert.equal(message.type, MESSAGE_TYPES.ERROR);
    assert.equal(message.error.code, "unsupported_mode");
    assert.match(message.error.message, /no supported entries/);
});

test("runtime uses runner-provided analyze preset capabilities", async () => {
    const message = await handleRunnerMessage(
        createRunMessage({
            requestId: "run-preset-cap",
            mode: "analyze",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
                preset: "health",
            },
        }),
        {
            runner: createStubRunner(),
            runnerCapabilities: {
                modes: ["analyze"],
                analyzePresets: ["receipt"],
            },
        }
    );

    assert.equal(message.type, MESSAGE_TYPES.ERROR);
    assert.equal(message.error.code, "unsupported_preset");
    assert.match(message.error.message, /receipt/);
});

test("runtime treats explicit empty analyze preset capabilities as unsupported", async () => {
    const message = await handleRunnerMessage(
        createRunMessage({
            requestId: "run-preset-empty-cap",
            mode: "analyze",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
                preset: "receipt",
            },
        }),
        {
            runner: createStubRunner(),
            runnerCapabilities: {
                modes: ["analyze"],
                analyzePresets: [],
            },
        }
    );

    assert.equal(message.type, MESSAGE_TYPES.ERROR);
    assert.equal(message.error.code, "unsupported_preset");
    assert.match(message.error.message, /no supported entries/);
});

test("runtime reports boot failures before capability checks", async () => {
    const message = await handleRunnerMessage(
        createRunMessage({
            requestId: "run-boot-error",
            mode: "export",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
            },
        }),
        {
            runner: createStubRunner(),
            runnerCapabilities: {
                modes: ["analyze", "export"],
                analyzePresets: ["receipt", "estimate"],
            },
            bootError: new Error("deterministic boot failure"),
        }
    );

    assert.equal(message.type, MESSAGE_TYPES.ERROR);
    assert.equal(message.error.code, "wasm_boot_failed");
    assert.match(message.error.message, /deterministic boot failure/);
});

test("runtime reserves cancel without promising it", async () => {
    const message = await handleRunnerMessage(createCancelMessage("run-7"));

    assert.equal(message.type, MESSAGE_TYPES.ERROR);
    assert.equal(message.requestId, "run-7");
    assert.equal(message.error.code, "cancel_unavailable");
});

test("runtime extracts error codes from structured runner errors", async () => {
    const runner = {
        runExport() {
            throw new Error("[invalid_settings] Cannot use both paths and inputs");
        },
    };

    const message = await handleRunnerMessage(
        createRunMessage({
            requestId: "run-err-code",
            mode: "export",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
            },
        }),
        { runner }
    );

    assert.equal(message.type, MESSAGE_TYPES.ERROR);
    assert.equal(message.error.code, "invalid_settings");
    assert.equal(message.error.message, "Cannot use both paths and inputs");
});

test("runtime extracts error codes from un-stringified error objects", async () => {
    const runner = {
        runExport() {
            throw { code: "custom_obj_err", message: "Inner object message" };
        },
    };

    const message = await handleRunnerMessage(
        createRunMessage({
            requestId: "run-err-code-obj",
            mode: "export",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
            },
        }),
        { runner }
    );

    assert.equal(message.type, MESSAGE_TYPES.ERROR);
    assert.equal(message.error.code, "custom_obj_err");
    assert.equal(message.error.message, "Inner object message");
});

test("runtime extracts error codes from fallback string errors", async () => {
    const runner = {
        runExport() {
            throw "[unknown_mode] What is this?";
        },
    };

    const message = await handleRunnerMessage(
        createRunMessage({
            requestId: "run-err-code-str",
            mode: "export",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
            },
        }),
        { runner }
    );

    assert.equal(message.type, MESSAGE_TYPES.ERROR);
    assert.equal(message.error.code, "unknown_mode");
    assert.equal(message.error.message, "What is this?");
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
