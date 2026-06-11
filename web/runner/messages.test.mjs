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



});
