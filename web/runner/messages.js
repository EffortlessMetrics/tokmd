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
    return Boolean(
        value &&
            value.type === MESSAGE_TYPES.RUN &&
            typeof value.requestId === "string" &&
            typeof value.mode === "string" &&
            value.args &&
            typeof value.args === "object" &&
            !Array.isArray(value.args) &&
            (() => {
                const hasValidRootInputs = Array.isArray(value.args.inputs) && value.args.inputs.every(isInMemoryInput);
                const hasValidScanInputs = value.args.scan && Array.isArray(value.args.scan.inputs) && value.args.scan.inputs.every(isInMemoryInput);
                const hasValidRootPaths = Array.isArray(value.args.paths) && value.args.paths.every(p => typeof p === "string");
                const hasValidScanPaths = value.args.scan && Array.isArray(value.args.scan.paths) && value.args.scan.paths.every(p => typeof p === "string");

                const isImplicitEmpty = value.args.inputs === undefined &&
                    value.args.paths === undefined &&
                    (!value.args.scan || (value.args.scan.inputs === undefined && value.args.scan.paths === undefined));

                const hasInvalidRootPaths = value.args.paths !== undefined && !hasValidRootPaths;
                const hasInvalidScanPaths = value.args.scan && value.args.scan.paths !== undefined && !hasValidScanPaths;

                if (hasInvalidRootPaths || hasInvalidScanPaths) {
                    return false;
                }

                return hasValidRootInputs ||
                    hasValidScanInputs ||
                    hasValidRootPaths ||
                    hasValidScanPaths ||
                    isImplicitEmpty;
            })()
    );
}

export function isCancelMessage(value) {
    return Boolean(
        value &&
            value.type === MESSAGE_TYPES.CANCEL &&
            typeof value.requestId === "string"
    );
}
