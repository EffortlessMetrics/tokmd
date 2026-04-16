import {
    MESSAGE_TYPES,
    SUPPORTED_ANALYZE_PRESETS,
    SUPPORTED_MODES,
    createErrorMessage,
    createResultMessage,
    normalizeAnalyzePreset,
    isCancelMessage,
    isRunMessage,
} from "./messages.js";

function asStringArray(value) {
    if (!Array.isArray(value)) {
        return [];
    }

    return value.filter((entry) => typeof entry === "string");
}

function resolveSupportedList(values, fallback) {
    const configured = asStringArray(values);

    return configured.length > 0 || Array.isArray(values) ? configured : fallback;
}

function formatSupportedList(values) {
    return values.length > 0 ? values.join(", ") : "no supported entries";
}

function extractRunnerError(error) {
    let message = "unknown runner error";
    let code = "run_failed";

    if (error instanceof Error && typeof error.message === "string") {
        message = error.message;
    } else if (error && typeof error.message === "string") {
        message = error.message;
        if (typeof error.code === "string") {
            code = error.code;
        }
    } else if (typeof error === "string") {
        message = error;
    }

    const match = message.match(/^\[([^\]]+)\]\s*(.*)$/);
    if (match) {
        return { code: match[1], message: match[2] || message };
    }

    return { code, message };
}

async function invokeRunner(runner, mode, args) {
    switch (mode) {
        case "lang":
            return runner.runLang(args);
        case "module":
            return runner.runModule(args);
        case "export":
            return runner.runExport(args);
        case "analyze":
            return runner.runAnalyze(args);
        default:
            throw new Error(`unsupported mode ${JSON.stringify(mode)}`);
    }
}

export async function handleRunnerMessage(message, options = {}) {
    const {
        runner = null,
        bootError = null,
        runnerCapabilities = null,
    } = options;

    if (isCancelMessage(message)) {
        return createErrorMessage(
            message.requestId,
            "cancel_unavailable",
            "browser runner reserves cancel, but tokmd-wasm cancellation is not wired yet"
        );
    }

    if (!isRunMessage(message)) {
        const requestId =
            message && typeof message.requestId === "string" ? message.requestId : null;
        return createErrorMessage(
            requestId,
            "invalid_message",
            "expected { type: \"run\", requestId, mode, args }"
        );
    }

    if (bootError) {
        return createErrorMessage(
            message.requestId,
            "wasm_boot_failed",
            `browser runner failed to initialize tokmd-wasm: ${extractRunnerError(bootError).message}`
        );
    }

    const hasExplicitRunnerCapabilities = runnerCapabilities !== null;
    const supportedModes = hasExplicitRunnerCapabilities
        ? resolveSupportedList(runnerCapabilities?.modes, [])
        : SUPPORTED_MODES;
    const supportedPresets = hasExplicitRunnerCapabilities
        ? resolveSupportedList(runnerCapabilities?.analyzePresets, [])
        : SUPPORTED_ANALYZE_PRESETS;

    if (!supportedModes.includes(message.mode)) {
        return createErrorMessage(
            message.requestId,
            "unsupported_mode",
            `browser runner supports only ${formatSupportedList(supportedModes)}; got ${JSON.stringify(message.mode)}`
        );
    }

    if (message.mode === "analyze") {
        const preset = normalizeAnalyzePreset(message.args);

        if (!supportedPresets.includes(preset)) {
            return createErrorMessage(
                message.requestId,
                "unsupported_preset",
                `browser runner supports analyze with ${formatSupportedList(supportedPresets)}; got ${JSON.stringify(preset)}`
            );
        }
    }

    if (!runner) {
        return createErrorMessage(
            message.requestId,
            "runner_unavailable",
            "browser runner is not ready yet"
        );
    }

    try {
        const data = await invokeRunner(runner, message.mode, message.args);
        return createResultMessage(message.requestId, data);
    } catch (error) {
        const extracted = extractRunnerError(error);
        return createErrorMessage(
            message.requestId,
            extracted.code,
            extracted.message
        );
    }
}

export function isProtocolMessage(value) {
    return Boolean(
        value &&
            typeof value === "object" &&
            typeof value.type === "string" &&
            Object.values(MESSAGE_TYPES).includes(value.type)
    );
}
