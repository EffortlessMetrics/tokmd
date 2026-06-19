export const RUNNER_PROTOCOL_VERSION = 2;

export const MESSAGE_TYPES = Object.freeze({
    READY: "ready",
    RUN: "run",
    PROGRESS: "progress",
    RESULT: "result",
    ERROR: "error",
    CANCEL: "cancel",
});

export const SUPPORTED_MODES = Object.freeze([
    "lang",
    "module",
    "export",
    "analyze",
]);

export const SUPPORTED_ANALYZE_PRESETS = Object.freeze(["receipt", "estimate"]);

export function normalizeAnalyzePreset(args = {}) {
    const candidate = args?.analyze?.preset ?? args?.preset ?? "receipt";

    if (typeof candidate !== "string") {
        return "receipt";
    }

    const normalized = candidate.trim().toLowerCase();
    return normalized || "receipt";
}

export function createReadyMessage(options = {}) {
    const { capabilities = {}, engine = null } = options;

    return {
        type: MESSAGE_TYPES.READY,
        protocolVersion: RUNNER_PROTOCOL_VERSION,
        capabilities: {
            modes: [...SUPPORTED_MODES],
            analyzePresets: [...SUPPORTED_ANALYZE_PRESETS],
            wasm: false,
            zipball: false,
            progress: false,
            cancel: false,
            downloads: false,
            ...capabilities,
        },
        ...(engine ? { engine } : {}),
    };
}

export function createRunMessage({ requestId, mode, args = {} }) {
    return {
        type: MESSAGE_TYPES.RUN,
        requestId,
        mode,
        args,
    };
}

export function createCancelMessage(requestId) {
    return {
        type: MESSAGE_TYPES.CANCEL,
        requestId,
    };
}

export function createProgressMessage(requestId, phase, options = {}) {
    const { mode = null, message = null, current = null, total = null } = options;
    const progress = {
        type: MESSAGE_TYPES.PROGRESS,
        requestId,
        phase,
    };

    if (mode) {
        progress.mode = mode;
    }

    if (message) {
        progress.message = message;
    }

    if (Number.isFinite(current)) {
        progress.current = current;
    }

    if (Number.isFinite(total)) {
        progress.total = total;
    }

    return progress;
}

export function createResultMessage(requestId, data) {
    return {
        type: MESSAGE_TYPES.RESULT,
        requestId,
        data,
    };
}

export function createErrorMessage(requestId, code, message) {
    return {
        type: MESSAGE_TYPES.ERROR,
        requestId: requestId ?? null,
        error: {
            code,
            message,
        },
    };
}

function isPlainObject(value) {
    return Boolean(value && typeof value === "object" && !Array.isArray(value));
}

export function isRunMessage(value) {
    return Boolean(
        value &&
            value.type === MESSAGE_TYPES.RUN &&
            typeof value.requestId === "string" &&
            typeof value.mode === "string" &&
            (!value.args || isPlainObject(value.args))
    );
}

export function isCancelMessage(value) {
    return Boolean(
        value &&
            value.type === MESSAGE_TYPES.CANCEL &&
            typeof value.requestId === "string"
    );
}
