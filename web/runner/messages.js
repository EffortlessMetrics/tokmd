export const RUNNER_PROTOCOL_VERSION = 1;

export const MESSAGE_TYPES = Object.freeze({
    READY: "ready",
    RUN: "run",
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

export function isInMemoryInput(value) {
    if (!value || typeof value !== "object") {
        return false;
    }

    if (typeof value.path !== "string" || value.path.trim().length === 0) {
        return false;
    }

    const hasText = typeof value.text === "string";
    const hasBase64 = typeof value.base64 === "string";

    return (hasText && !hasBase64) || (!hasText && hasBase64);
}

export function isRunMessage(value) {
    if (!value || value.type !== MESSAGE_TYPES.RUN || typeof value.requestId !== "string" || typeof value.mode !== "string") {
        return false;
    }
    if (!value.args || typeof value.args !== "object" || Array.isArray(value.args)) {
        return false;
    }
    if (Array.isArray(value.args.inputs)) {
        return value.args.inputs.every(isInMemoryInput);
    } else if (Array.isArray(value.args.paths)) {
        return value.args.paths.every(p => typeof p === "string");
    } else if (value.args.scan && typeof value.args.scan === "object") {
        return true;
    }
    return false;
}

export function isCancelMessage(value) {
    return Boolean(
        value &&
            value.type === MESSAGE_TYPES.CANCEL &&
            typeof value.requestId === "string"
    );
}
