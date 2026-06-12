import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

import { createCancelMessage, createRunMessage, MESSAGE_TYPES } from "./messages.js";
import { handleRunnerMessage, isProtocolMessage } from "./runtime.js";

function wasmCapabilityMatrix() {
    return JSON.parse(
        readFileSync(
            new URL("../../docs/capabilities/wasm.json", import.meta.url),
            "utf8"
        )
    );
}

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

test("runtime rejects native-only modes before runner execution", async () => {
    const message = await handleRunnerMessage(
        createRunMessage({
            requestId: "run-native-mode",
            mode: "gate",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
            },
        }),
        { runner: createStubRunner() }
    );

    assert.equal(message.type, MESSAGE_TYPES.ERROR);
    assert.equal(message.error.code, "unsupported_mode");
    assert.equal(message.requestId, "run-native-mode");
});

test("runtime rejects every native-only matrix command", async () => {
    const matrix = wasmCapabilityMatrix();
    const nativeOnlyCommands = Object.entries(matrix.commands)
        .filter(([, capabilities]) => capabilities.native_only === true)
        .map(([command]) => command);

    assert.ok(nativeOnlyCommands.length > 0);

    for (const mode of nativeOnlyCommands) {
        const message = await handleRunnerMessage(
            createRunMessage({
                requestId: `native-${mode}`,
                mode,
                args: {
                    inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
                },
            }),
            { runner: createStubRunner() }
        );

        assert.equal(message.type, MESSAGE_TYPES.ERROR, mode);
        assert.equal(message.error.code, "unsupported_mode", mode);
    }
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

test("runtime extracts error codes from duck-typed error objects", async () => {
    const runner = {
        runExport() {
            // eslint-disable-next-line no-throw-literal
            throw { message: "duck typed object error", code: "duck_code" };
        },
    };

    const message = await handleRunnerMessage(
        createRunMessage({
            requestId: "run-err-duck-code",
            mode: "export",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
            },
        }),
        { runner }
    );

    assert.equal(message.type, MESSAGE_TYPES.ERROR);
    assert.equal(message.error.code, "duck_code");
    assert.equal(message.error.message, "duck typed object error");
});

test("runtime extracts error codes from Error instances", async () => {
    const runner = {
        runExport() {
            const error = new Error("typed error instance");
            error.code = "typed_code";
            throw error;
        },
    };

    const message = await handleRunnerMessage(
        createRunMessage({
            requestId: "run-err-error-code",
            mode: "export",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
            },
        }),
        { runner }
    );

    assert.equal(message.type, MESSAGE_TYPES.ERROR);
    assert.equal(message.error.code, "typed_code");
    assert.equal(message.error.message, "typed error instance");
});

test("runtime extracts messages from duck-typed error objects without codes", async () => {
    const runner = {
        runExport() {
            // eslint-disable-next-line no-throw-literal
            throw { message: "duck typed message only" };
        },
    };

    const message = await handleRunnerMessage(
        createRunMessage({
            requestId: "run-err-duck-no-code",
            mode: "export",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
            },
        }),
        { runner }
    );

    assert.equal(message.type, MESSAGE_TYPES.ERROR);
    assert.equal(message.error.code, "run_failed");
    assert.equal(message.error.message, "duck typed message only");
});

test("runtime applies bracket format codes over duck-typed codes", async () => {
    const runner = {
        runExport() {
            // eslint-disable-next-line no-throw-literal
            throw { message: "[bracket_code] bracket message", code: "duck_code" };
        },
    };

    const message = await handleRunnerMessage(
        createRunMessage({
            requestId: "run-err-duck-bracket",
            mode: "export",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
            },
        }),
        { runner }
    );

    assert.equal(message.type, MESSAGE_TYPES.ERROR);
    assert.equal(message.error.code, "bracket_code");
    assert.equal(message.error.message, "bracket message");
});

test("runtime returns results once a runner is available", async () => {
    const progress = [];
    const message = await handleRunnerMessage(
        createRunMessage({
            requestId: "run-8",
            mode: "lang",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
            },
        }),
        {
            runner: createStubRunner(),
            onProgress: (update) => progress.push(update),
        }
    );

    assert.equal(message.type, MESSAGE_TYPES.RESULT);
    assert.equal(message.requestId, "run-8");
    assert.equal(message.data.mode, "lang");
    assert.equal(message.data.total.files, 1);
    assert.deepEqual(
        progress.map((update) => update.phase),
        ["start", "fetch", "done"]
    );
    assert.deepEqual(
        progress.map((update) => update.type),
        [MESSAGE_TYPES.PROGRESS, MESSAGE_TYPES.PROGRESS, MESSAGE_TYPES.PROGRESS]
    );
});

test("analyze without preset defaults to receipt and returns a result", async () => {
    const progress = [];
    const message = await handleRunnerMessage(
        createRunMessage({
            requestId: "run-9",
            mode: "analyze",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
            },
        }),
        {
            runner: createStubRunner(),
            onProgress: (update) => progress.push(update),
        }
    );

    assert.equal(message.type, MESSAGE_TYPES.RESULT);
    assert.equal(message.requestId, "run-9");
    assert.equal(message.data.mode, "analysis");
    assert.equal(message.data.preset, "receipt");
    assert.deepEqual(
        progress.map((update) => update.phase),
        ["start", "fetch", "analyze", "done"]
    );
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

test("runtime emits error progress when runner execution fails", async () => {
    const progress = [];
    const runner = {
        runExport() {
            throw new Error("[invalid_settings] Cannot use both paths and inputs");
        },
    };

    const message = await handleRunnerMessage(
        createRunMessage({
            requestId: "run-progress-error",
            mode: "export",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
            },
        }),
        {
            runner,
            onProgress: (update) => progress.push(update),
        }
    );

    assert.equal(message.type, MESSAGE_TYPES.ERROR);
    assert.equal(message.error.code, "invalid_settings");
    assert.deepEqual(
        progress.map((update) => update.phase),
        ["start", "fetch", "error"]
    );
    assert.match(progress.at(-1).message, /Cannot use both paths and inputs/);
});

test("runtime emits analyze progress before analyze runner failures", async () => {
    const progress = [];
    const runner = {
        runAnalyze() {
            throw new Error("[analysis_failed] analysis exploded");
        },
    };

    const message = await handleRunnerMessage(
        createRunMessage({
            requestId: "run-analyze-progress-error",
            mode: "analyze",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
            },
        }),
        {
            runner,
            onProgress: (update) => progress.push(update),
        }
    );

    assert.equal(message.type, MESSAGE_TYPES.ERROR);
    assert.equal(message.error.code, "analysis_failed");
    assert.deepEqual(
        progress.map((update) => update.phase),
        ["start", "fetch", "analyze", "error"]
    );
    assert.match(progress.at(-1).message, /analysis exploded/);
});

test("runtime delegates invalid inputs shape to runner and extracts invalid_settings", async () => {
    // createStubRunner doesn't validate inputs, so we simulate the WASM runner failure
    const runner = {
        runLang() {
            throw new Error("[invalid_settings] a non-empty relative file path");
        },
    };

    const message = await handleRunnerMessage(
        createRunMessage({
            requestId: "run-invalid",
            mode: "lang",
            args: { inputs: [{ path: "", text: "bad\n" }] },
        }),
        { runner }
    );

    assert.equal(message.type, MESSAGE_TYPES.ERROR);
    assert.equal(message.error.code, "invalid_settings");
    assert.equal(message.requestId, "run-invalid");
});
