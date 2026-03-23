const DEFAULT_LIMITS = Object.freeze({
    maxFiles: 32,
    maxBytes: 750_000,
    maxFileBytes: 120_000,
});

const VENDOR_SEGMENTS = new Set([
    ".git",
    ".next",
    ".nuxt",
    "build",
    "coverage",
    "dist",
    "node_modules",
    "target",
    "vendor",
]);

const BINARY_EXTENSIONS = new Set([
    "7z",
    "a",
    "bin",
    "bmp",
    "class",
    "dll",
    "dylib",
    "eot",
    "exe",
    "gif",
    "gz",
    "ico",
    "jar",
    "jpeg",
    "jpg",
    "lockb",
    "mp3",
    "mp4",
    "o",
    "otf",
    "pdf",
    "png",
    "pyc",
    "pyo",
    "so",
    "tar",
    "ttf",
    "wasm",
    "webm",
    "webp",
    "woff",
    "woff2",
    "xz",
    "zip",
]);

const ROOT_PREFERRED_FILES = new Set([
    "cargo.toml",
    "go.mod",
    "package.json",
    "pyproject.toml",
    "readme.md",
    "requirements.txt",
]);

const PRIMARY_PREFIXES = [
    "app/",
    "crates/",
    "lib/",
    "packages/",
    "pkg/",
    "src/",
    "web/",
];

const SECONDARY_PREFIXES = [
    "cmd/",
    "docs/",
    "examples/",
    "internal/",
    "scripts/",
    "tests/",
];

const repoCache = new Map();
const GITHUB_SEGMENT_PATTERN = /^[A-Za-z0-9_.-]+$/;

function normalizePath(value) {
    return value.replaceAll("\\", "/");
}

function pathExtension(path) {
    const lastSegment = path.split("/").at(-1) ?? "";
    const dotIndex = lastSegment.lastIndexOf(".");
    return dotIndex === -1 ? "" : lastSegment.slice(dotIndex + 1).toLowerCase();
}

function pathPriority(path) {
    const normalized = normalizePath(path);
    const lower = normalized.toLowerCase();
    const lastSegment = lower.split("/").at(-1) ?? "";

    if (!lower.includes("/") && ROOT_PREFERRED_FILES.has(lastSegment)) {
        return 0;
    }

    if (PRIMARY_PREFIXES.some((prefix) => lower.startsWith(prefix))) {
        return 1;
    }

    if (SECONDARY_PREFIXES.some((prefix) => lower.startsWith(prefix))) {
        return 2;
    }

    if (lower.startsWith(".")) {
        return 4;
    }

    if (!lower.includes("/") && lastSegment.startsWith("_")) {
        return 5;
    }

    return 3;
}

function isLikelyVendorPath(path) {
    return normalizePath(path)
        .toLowerCase()
        .split("/")
        .some((segment) => VENDOR_SEGMENTS.has(segment));
}

function isLikelyBinaryPath(path) {
    return BINARY_EXTENSIONS.has(pathExtension(path));
}

function decodeText(bytes) {
    if (bytes.includes(0)) {
        return null;
    }

    try {
        return new TextDecoder("utf-8", { fatal: true }).decode(bytes);
    } catch {
        return null;
    }
}

function normalizePositiveLimit(name, value, fallback) {
    if (value === undefined || value === null) {
        return fallback;
    }

    const numeric = Number(value);
    if (!Number.isFinite(numeric) || numeric <= 0) {
        throw new Error(`${name} must be a positive number`);
    }

    return Math.floor(numeric);
}

function normalizeLimits(options = {}) {
    return {
        maxFiles: normalizePositiveLimit("maxFiles", options.maxFiles, DEFAULT_LIMITS.maxFiles),
        maxBytes: normalizePositiveLimit("maxBytes", options.maxBytes, DEFAULT_LIMITS.maxBytes),
        maxFileBytes: normalizePositiveLimit(
            "maxFileBytes",
            options.maxFileBytes,
            DEFAULT_LIMITS.maxFileBytes
        ),
    };
}

function normalizeToken(value) {
    if (typeof value !== "string") {
        return null;
    }

    const trimmed = value.trim();
    return trimmed ? trimmed : null;
}

function buildCacheKey({ owner, repo, ref, limits, token }) {
    return JSON.stringify({
        owner,
        repo,
        ref,
        auth: token ? `token:${token}` : "anonymous",
        ...limits,
    });
}

function buildGitHubHeaders(accept, token) {
    const headers = { Accept: accept };

    if (token) {
        headers.Authorization = `Bearer ${token}`;
    }

    return headers;
}

function rawContentHeaders(token) {
    return buildGitHubHeaders("application/vnd.github.raw+json", token);
}

function githubJsonHeaders(token) {
    return buildGitHubHeaders("application/vnd.github+json", token);
}

function createIngestError(code, message, extra = {}) {
    const error = new Error(message);
    error.code = code;
    return Object.assign(error, extra);
}

function createAbortError() {
    return createIngestError("repo_load_aborted", "GitHub repo load was canceled", {
        name: "AbortError",
    });
}

function normalizeAbortReason(error, signal) {
    if (
        signal?.aborted ||
        error?.name === "AbortError" ||
        error?.code === "ABORT_ERR"
    ) {
        return createAbortError();
    }

    return error;
}

function throwIfAborted(signal) {
    if (signal?.aborted) {
        throw createAbortError();
    }
}

function withAbortSignal(promise, signal) {
    if (!signal) {
        return promise;
    }

    throwIfAborted(signal);

    return new Promise((resolve, reject) => {
        const onAbort = () => {
            cleanup();
            reject(createAbortError());
        };
        const cleanup = () => {
            signal.removeEventListener("abort", onAbort);
        };

        signal.addEventListener("abort", onAbort, { once: true });

        promise.then(
            (value) => {
                cleanup();
                resolve(value);
            },
            (error) => {
                cleanup();
                reject(normalizeAbortReason(error, signal));
            }
        );
    });
}

function parseRetryAfterSeconds(headers) {
    const raw = headers.get("retry-after");
    if (!raw) {
        return null;
    }

    const value = Number(raw);
    return Number.isFinite(value) && value >= 0 ? value : null;
}

function parseRateLimitResetAt(headers) {
    const raw = headers.get("x-ratelimit-reset");
    if (!raw) {
        return null;
    }

    const epochSeconds = Number(raw);
    return Number.isFinite(epochSeconds) && epochSeconds > 0
        ? new Date(epochSeconds * 1000).toISOString()
        : null;
}

async function readResponseMessage(response) {
    try {
        const text = await response.text();
        if (!text) {
            return "";
        }

        try {
            const parsed = JSON.parse(text);
            return typeof parsed?.message === "string" ? parsed.message : text;
        } catch {
            return text;
        }
    } catch {
        return "";
    }
}

function emitProgress(callback, update) {
    if (typeof callback === "function") {
        callback(update);
    }
}

function withCacheHit(result) {
    return {
        ...result,
        ingest: {
            ...result.ingest,
            cache: {
                ...result.ingest.cache,
                hit: true,
            },
        },
    };
}

function buildPartialReasons({
    treeTruncated,
    skippedTooLarge,
    skippedBudget,
    skippedFileLimit,
}) {
    const reasons = [];

    if (treeTruncated) {
        reasons.push({
            code: "tree_truncated",
            message: "GitHub truncated the recursive tree listing; only part of the repo may be loaded.",
        });
    }

    if (skippedTooLarge > 0) {
        reasons.push({
            code: "too_large_files",
            message: `${skippedTooLarge} file(s) exceeded the per-file byte limit and were skipped.`,
        });
    }

    if (skippedBudget > 0) {
        reasons.push({
            code: "byte_budget",
            message: `${skippedBudget} file(s) were skipped after the total byte budget was reached.`,
        });
    }

    if (skippedFileLimit > 0) {
        reasons.push({
            code: "file_limit",
            message: `${skippedFileLimit} candidate file(s) were not loaded after reaching the file limit.`,
        });
    }

    return reasons;
}

async function fetchWithRateLimitMessage(fetchImpl, url, options = {}) {
    throwIfAborted(options.signal);

    const response = await withAbortSignal(fetchImpl(url, options), options.signal);
    if (response.ok) {
        return response;
    }

    const authMode = options.authMode ?? "anonymous";
    const message = await readResponseMessage(response);
    const remaining = response.headers.get("x-ratelimit-remaining");
    const retryAfterSeconds = parseRetryAfterSeconds(response.headers);
    const resetAt = parseRateLimitResetAt(response.headers);
    const secondaryLimited =
        response.status === 429 ||
        retryAfterSeconds !== null ||
        /secondary rate limit/i.test(message);

    if (response.status === 401) {
        throw createIngestError(
            "github_auth_required",
            authMode === "token"
                ? "GitHub rejected the supplied token for this repo load."
                : "GitHub repo access requires a token or a public repository.",
            { status: response.status }
        );
    }

    if (response.status === 404) {
        throw createIngestError(
            "github_repo_unavailable",
            authMode === "token"
                ? "GitHub repo or ref was not found for the supplied token."
                : "GitHub repo or ref was not found, or it requires a token.",
            { status: response.status }
        );
    }

    if (response.status === 403 && remaining === "0") {
        throw createIngestError(
            "github_primary_rate_limit",
            resetAt
                ? `GitHub API rate limit reached. Try again after ${resetAt}.`
                : "GitHub API rate limit reached for browser repo fetches.",
            {
                status: response.status,
                resetAt,
            }
        );
    }

    if (secondaryLimited) {
        throw createIngestError(
            "github_secondary_rate_limit",
            retryAfterSeconds !== null
                ? `GitHub asked the browser runner to slow down. Retry after ${retryAfterSeconds}s.`
                : message || "GitHub temporarily rate-limited this browser repo load.",
            {
                status: response.status,
                retryAfterSeconds,
            }
        );
    }

    throw createIngestError(
        "github_request_failed",
        `GitHub request failed: ${response.status} ${response.statusText}`,
        {
            status: response.status,
            responseMessage: message,
        }
    );
}

async function fetchRepositoryTree(fetchImpl, owner, repo, ref, token, signal) {
    const url =
        `https://api.github.com/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}` +
        `/git/trees/${encodeURIComponent(ref)}?recursive=1`;
    const response = await fetchWithRateLimitMessage(fetchImpl, url, {
        authMode: token ? "token" : "anonymous",
        headers: githubJsonHeaders(token),
        signal,
    });
    return await withAbortSignal(response.json(), signal);
}

async function fetchFileBytes(fetchImpl, owner, repo, ref, path, token, signal) {
    const encodedPath = normalizePath(path)
        .split("/")
        .map((segment) => encodeURIComponent(segment))
        .join("/");
    const url =
        `https://api.github.com/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}` +
        `/contents/${encodedPath}?ref=${encodeURIComponent(ref)}`;
    const response = await fetchWithRateLimitMessage(fetchImpl, url, {
        authMode: token ? "token" : "anonymous",
        headers: rawContentHeaders(token),
        signal,
    });
    return new Uint8Array(await withAbortSignal(response.arrayBuffer(), signal));
}

export function parseGitHubRepo(value) {
    if (typeof value !== "string") {
        throw new Error("GitHub repository must be a string like owner/repo");
    }

    const normalized = value.trim();
    if (!normalized) {
        throw new Error("GitHub repository must not be empty");
    }

    const fromSegments = (segments) => {
        if (segments.length !== 2) {
            throw new Error("GitHub repository must look like owner/repo");
        }

        const owner = segments[0];
        const repo = segments[1].replace(/\.git$/i, "");

        if (!owner || !repo) {
            throw new Error("GitHub repository must look like owner/repo");
        }

        if (!GITHUB_SEGMENT_PATTERN.test(owner) || !GITHUB_SEGMENT_PATTERN.test(repo)) {
            throw new Error("GitHub repository must look like owner/repo");
        }

        return { owner, repo };
    };

    if (/^https?:\/\//i.test(normalized)) {
        const url = new URL(normalized);
        if (!["github.com", "www.github.com"].includes(url.hostname.toLowerCase())) {
            throw new Error("GitHub repository URL must point to github.com");
        }

        return fromSegments(url.pathname.split("/").filter(Boolean));
    }

    if (normalized.includes(":") || normalized.includes("?") || normalized.includes("#")) {
        throw new Error("GitHub repository must look like owner/repo");
    }

    return fromSegments(normalized.replace(/\/+$/g, "").split("/").filter(Boolean));
}

export function selectGitHubTreeEntries(entries, options = {}) {
    const limits = normalizeLimits(options);
    const stats = {
        treeEntries: Array.isArray(entries) ? entries.length : 0,
        blobsSeen: 0,
        skippedVendor: 0,
        skippedBinaryPath: 0,
        skippedTooLarge: 0,
        skippedFileLimit: 0,
    };
    const selected = [];

    const orderedEntries = [...(entries ?? [])]
        .filter((entry) => entry?.type === "blob" && typeof entry.path === "string")
        .sort((left, right) => {
            const leftPath = normalizePath(left.path);
            const rightPath = normalizePath(right.path);
            const priority = pathPriority(leftPath) - pathPriority(rightPath);
            return priority === 0 ? leftPath.localeCompare(rightPath) : priority;
        });

    for (const entry of orderedEntries) {
        stats.blobsSeen += 1;
        const path = normalizePath(entry.path);

        if (isLikelyVendorPath(path)) {
            stats.skippedVendor += 1;
            continue;
        }

        if (isLikelyBinaryPath(path)) {
            stats.skippedBinaryPath += 1;
            continue;
        }

        if (typeof entry.size === "number" && entry.size > limits.maxFileBytes) {
            stats.skippedTooLarge += 1;
            continue;
        }

        selected.push({
            path,
            size: typeof entry.size === "number" ? entry.size : null,
        });
    }

    return {
        selected,
        stats,
        limits,
    };
}

export function clearGitHubRepoCache() {
    repoCache.clear();
}

export async function fetchGitHubRepoInputs(options = {}) {
    const { owner, repo } = parseGitHubRepo(options.repo);
    const ref = typeof options.ref === "string" && options.ref.trim() ? options.ref.trim() : "main";
    const limits = normalizeLimits(options);
    const fetchImpl = options.fetchImpl ?? fetch;
    const token = normalizeToken(options.token);
    const signal = options.signal;
    const onProgress = options.onProgress;
    const cacheKey = buildCacheKey({ owner, repo, ref, limits, token });

    throwIfAborted(signal);

    if (repoCache.has(cacheKey)) {
        const cached = withCacheHit(await withAbortSignal(repoCache.get(cacheKey), signal));
        emitProgress(onProgress, {
            phase: "cache",
            current: 1,
            total: 1,
            message: `Using in-memory cache for ${owner}/${repo}@${ref}`,
        });
        emitProgress(onProgress, {
            phase: "complete",
            current: cached.ingest.loadedFiles,
            total: cached.ingest.loadedFiles,
            loadedFiles: cached.ingest.loadedFiles,
            message: `Loaded ${cached.ingest.loadedFiles} file(s) from ${owner}/${repo}@${ref} (cache)`,
        });
        return cached;
    }

    const loadPromise = (async () => {
        emitProgress(onProgress, {
            phase: "tree",
            current: 0,
            total: 1,
            message: `Fetching GitHub tree for ${owner}/${repo}@${ref}`,
        });
        const tree = await fetchRepositoryTree(fetchImpl, owner, repo, ref, token, signal);
        const selection = selectGitHubTreeEntries(tree.tree, limits);
        const inputs = [];
        let bytesRead = 0;
        let skippedBinaryContent = 0;
        let skippedBudget = 0;
        const treeTruncated = Boolean(tree?.truncated);

        emitProgress(onProgress, {
            phase: "files",
            current: 0,
            total: selection.selected.length,
            loadedFiles: 0,
            message: `Loading candidate files for ${owner}/${repo}@${ref}`,
        });

        for (const [index, entry] of selection.selected.entries()) {
            throwIfAborted(signal);
            emitProgress(onProgress, {
                phase: "files",
                current: index + 1,
                total: selection.selected.length,
                loadedFiles: inputs.length,
                message: `Loading ${entry.path}`,
            });

            const bytes = await fetchFileBytes(fetchImpl, owner, repo, ref, entry.path, token, signal);

            if (bytes.length > limits.maxFileBytes) {
                selection.stats.skippedTooLarge += 1;
                continue;
            }

            if (bytesRead + bytes.length > limits.maxBytes) {
                skippedBudget += 1;
                continue;
            }

            const text = decodeText(bytes);
            if (text === null) {
                skippedBinaryContent += 1;
                continue;
            }

            bytesRead += bytes.length;
            inputs.push({
                path: entry.path,
                text,
            });

            if (inputs.length >= limits.maxFiles) {
                selection.stats.skippedFileLimit = selection.selected.length - index - 1;
                break;
            }
        }

        const partialReasons = buildPartialReasons({
            treeTruncated,
            skippedTooLarge: selection.stats.skippedTooLarge,
            skippedBudget,
            skippedFileLimit: selection.stats.skippedFileLimit,
        });
        const ingest = {
            bytesRead,
            loadedFiles: inputs.length,
            skippedBinaryContent,
            skippedBudget,
            partial: partialReasons.length > 0,
            partialReasons,
            treeEntriesTruncated: treeTruncated,
            cache: {
                scope: "memory",
                hit: false,
            },
            authMode: token ? "token" : "anonymous",
            ...selection.stats,
            ...limits,
        };

        if (inputs.length === 0) {
            throw createIngestError(
                "no_browser_safe_text",
                "No browser-safe text files remained after GitHub filtering and limits",
                { ingest }
            );
        }

        emitProgress(onProgress, {
            phase: "complete",
            current: inputs.length,
            total: inputs.length,
            loadedFiles: inputs.length,
            message: `Loaded ${inputs.length} file(s) from ${owner}/${repo}@${ref}`,
        });

        return {
            inputs,
            source: {
                repo: `${owner}/${repo}`,
                ref,
                strategy: "github-tree-contents",
            },
            ingest,
        };
    })();

    repoCache.set(cacheKey, loadPromise);

    try {
        return await loadPromise;
    } catch (error) {
        repoCache.delete(cacheKey);
        throw error;
    }
}
