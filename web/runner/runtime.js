import {
    MESSAGE_TYPES,
    SUPPORTED_ANALYZE_PRESETS,
    SUPPORTED_MODES,
    createErrorMessage,
    normalizeAnalyzePreset,
    isCancelMessage,
    isRunMessage,
} from "./messages.js";

export function handleRunnerMessage(message) {
    if (isCancelMessage(message)) {
        return createErrorMessage(
            message.requestId,
            "cancel_unavailable",
            "browser runner skeleton reserves cancel, but tokmd-wasm cancellation is not wired yet"
        );
    }

    if (!isRunMessage(message)) {
        return createErrorMessage(
            null,
            "invalid_message",
            "expected { type: \"run\", requestId, mode, args }"
        );
    }

    if (!SUPPORTED_MODES.includes(message.mode)) {
        return createErrorMessage(
            message.requestId,
            "unsupported_mode",
            `browser runner supports only ${SUPPORTED_MODES.join(", ")}; got ${JSON.stringify(message.mode)}`
        );
    }

    if (message.mode === "analyze") {
        const preset = normalizeAnalyzePreset(message.args);

        if (!SUPPORTED_ANALYZE_PRESETS.includes(preset)) {
            return createErrorMessage(
                message.requestId,
                "unsupported_preset",
                `browser runner supports analyze only with preset="receipt" or preset="estimate"; got ${JSON.stringify(preset)}`
            );
        }
    }

    return createErrorMessage(
        message.requestId,
        "runner_not_wired",
        `browser runner skeleton is ready, but tokmd-wasm bootstrap for mode ${JSON.stringify(message.mode)} is not wired yet`
    );
}

export function isProtocolMessage(value) {
    return Boolean(
        value &&
            typeof value === "object" &&
            typeof value.type === "string" &&
            Object.values(MESSAGE_TYPES).includes(value.type)
    );
}
