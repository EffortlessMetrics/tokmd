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

function hasOnlyKeys(value, allowedKeys) {
    return Object.keys(value).every((key) => allowedKeys.includes(key));
}

function isAnalyzeOptions(value) {
    return Boolean(
        value &&
            typeof value === "object" &&
            !Array.isArray(value) &&
            hasOnlyKeys(value, ["preset"]) &&
            (value.preset === undefined || typeof value.preset === "string")
    );
}

function isScanOptions(value) {
    return Boolean(value && typeof value === "object" && !Array.isArray(value));
}

function hasValidPayloadType(args) {
    const hasInputs = Array.isArray(args.inputs) && args.inputs.every(isInMemoryInput);
    const hasPaths = Array.isArray(args.paths) && args.paths.every(p => typeof p === "string");
    const hasScan = isScanOptions(args.scan);

    return hasInputs || hasPaths || hasScan;
}

function isRunArgsForMode(mode, args) {
    if (!args || typeof args !== "object" || Array.isArray(args)) {
        return false;
    }

    if (!hasValidPayloadType(args)) {
        return false;
    }

    const payloadKeys = [];
    if (args.inputs !== undefined) payloadKeys.push("inputs");
    if (args.paths !== undefined) payloadKeys.push("paths");
    if (args.scan !== undefined) payloadKeys.push("scan");

    if (mode === "analyze") {
        return Boolean(
            hasOnlyKeys(args, [...payloadKeys, "preset", "analyze"]) &&
                (args.preset === undefined || typeof args.preset === "string") &&
                (args.analyze === undefined || isAnalyzeOptions(args.analyze))
        );
    }

    if (mode === "lang") {
        return Boolean(
            hasOnlyKeys(args, [...payloadKeys, "files"]) &&
                (args.files === undefined || typeof args.files === "boolean")
        );
    }

    return hasOnlyKeys(args, payloadKeys);
}

export function isRunMessage(value) {
    return Boolean(
        value &&
            value.type === MESSAGE_TYPES.RUN &&
            typeof value.requestId === "string" &&
            typeof value.mode === "string" &&
            isRunArgsForMode(value.mode, value.args)
    );
}

export function isCancelMessage(value) {
    return Boolean(
        value &&
            value.type === MESSAGE_TYPES.CANCEL &&
            typeof value.requestId === "string"
    );
}
