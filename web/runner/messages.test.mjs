import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

import {
    MESSAGE_TYPES,
    RUNNER_PROTOCOL_VERSION,
    SUPPORTED_ANALYZE_PRESETS,
    SUPPORTED_MODES,
    createCancelMessage,
    createProgressMessage,
    createReadyMessage,
    createRunMessage,
    isCancelMessage,
    isInMemoryInput,
    isRunMessage,
    normalizeAnalyzePreset,
} from "./messages.js";

function wasmCapabilityMatrix() {
    return JSON.parse(
        readFileSync(
            new URL("../../docs/capabilities/wasm.json", import.meta.url),
            "utf8"
        )
    );
}

function isBrowserRunnable(capabilities) {
    return (
        (capabilities.browser_safe === true ||
            capabilities.browser_safe === "partial") &&
        capabilities.native_only === false
    );
}

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
    assert.equal(message.capabilities.progress, false);
});

test("supported modes stay aligned with the WASM capability matrix", () => {
    const matrix = wasmCapabilityMatrix();
    const commands = matrix.commands;
    const matrixModes = Object.entries(commands)
        .filter(([, capabilities]) => isBrowserRunnable(capabilities))
        .map(([command]) => command)
        .sort();

    assert.deepEqual([...SUPPORTED_MODES].sort(), matrixModes);

    for (const mode of SUPPORTED_MODES) {
        const capabilities = commands[mode];

        assert.ok(capabilities, `${mode} missing from WASM capability matrix`);
        assert.notEqual(capabilities.native_only, true);
        assert.ok(
            capabilities.rootless_safe === true ||
                capabilities.rootless_safe === "partial",
            `${mode} must be rootless-safe or partial in browser runner`
        );
    }

    for (const [command, capabilities] of Object.entries(commands)) {
        if (capabilities.native_only === true) {
            assert.equal(
                SUPPORTED_MODES.includes(command),
                false,
                `${command} is native-only and must not be a runner mode`
            );
        }
    }
});

test("supported analyze presets stay aligned with the WASM capability matrix", () => {
    const matrix = wasmCapabilityMatrix();
    const analyze = matrix.commands.analyze;

    assert.ok(analyze, "analyze missing from WASM capability matrix");
    assert.deepEqual(
        [...SUPPORTED_ANALYZE_PRESETS].sort(),
        [...analyze.browser_analyze_presets].sort()
    );
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

test("progress helper produces protocol messages with stable fields", () => {
    const message = createProgressMessage("run-1", "fetch", {
        mode: "lang",
        message: "Fetching in-memory inputs",
        current: 1,
        total: 3,
    });

    assert.deepEqual(message, {
        type: MESSAGE_TYPES.PROGRESS,
        requestId: "run-1",
        phase: "fetch",
        mode: "lang",
        message: "Fetching in-memory inputs",
        current: 1,
        total: 3,
    });
});

test("run messages require explicit in-memory inputs", () => {
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
            requestId: "files",
            mode: "lang",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
                files: true,
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
    assert.equal(
        isRunMessage({
            type: "run",
            requestId: "files-type",
            mode: "lang",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
                files: "yes",
            },
        }),
        false
    );
    assert.equal(
        isRunMessage({ type: "run", requestId: "x", mode: "lang", args: {} }),
        false
    );
    assert.equal(
        isRunMessage({
            type: "run",
            requestId: "x",
            mode: "lang",
            args: { paths: ["src/lib.rs"] },
        }),
        false
    );
    assert.equal(
        isRunMessage({
            type: "run",
            requestId: "x",
            mode: "lang",
            args: {
                scan: {
                    inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
                },
            },
        }),
        true
    );
    assert.equal(
        isRunMessage({
            type: "run",
            requestId: "x",
            mode: "lang",
            args: {
                inputs: [{ path: "root.rs", text: "pub fn root() {}\n" }],
                scan: {
                    inputs: [{ path: "nested.rs", text: "pub fn nested() {}\n" }],
                },
            },
        }),
        false
    );
    assert.equal(
        isRunMessage({
            type: "run",
            requestId: "x",
            mode: "lang",
            args: {
                scan: null,
            },
        }),
        false
    );
    assert.equal(
        isRunMessage({
            type: "run",
            requestId: "x",
            mode: "lang",
            args: {
                scan: {
                    inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
                    paths: ["src/lib.rs"],
                },
            },
        }),
        false
    );
    assert.equal(
        isRunMessage({
            type: "run",
            requestId: "x",
            mode: "lang",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
                extra: true,
            },
        }),
        false
    );
    assert.equal(
        isRunMessage({
            type: "run",
            requestId: "x",
            mode: "lang",
            args: { inputs: [{ path: "", text: "bad\n" }] },
        }),
        false
    );
});

test("analyze run messages allow only explicit preset options with inputs", () => {
    const inputs = [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }];

    assert.equal(
        isRunMessage({
            type: "run",
            requestId: "analyze-1",
            mode: "analyze",
            args: { inputs, preset: "estimate" },
        }),
        true
    );
    assert.equal(
        isRunMessage({
            type: "run",
            requestId: "analyze-2",
            mode: "analyze",
            args: { inputs, analyze: { preset: "receipt" } },
        }),
        true
    );
    assert.equal(
        isRunMessage({
            type: "run",
            requestId: "analyze-scan",
            mode: "analyze",
            args: { scan: { inputs }, preset: "estimate" },
        }),
        true
    );
    assert.equal(
        isRunMessage({
            type: "run",
            requestId: "analyze-3",
            mode: "analyze",
            args: { inputs, preset: 1 },
        }),
        false
    );
    assert.equal(
        isRunMessage({
            type: "run",
            requestId: "analyze-4",
            mode: "analyze",
            args: { inputs, analyze: { preset: "receipt", extra: true } },
        }),
        false
    );
});
