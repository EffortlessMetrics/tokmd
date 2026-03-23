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

function normalizeLimits(options = {}) {
    return {
        maxFiles: options.maxFiles ?? DEFAULT_LIMITS.maxFiles,
        maxBytes: options.maxBytes ?? DEFAULT_LIMITS.maxBytes,
        maxFileBytes: options.maxFileBytes ?? DEFAULT_LIMITS.maxFileBytes,
    };
}

function buildCacheKey({ owner, repo, ref, limits }) {
    return JSON.stringify({
        owner,
        repo,
        ref,
        ...limits,
    });
}

function rawContentHeaders() {
    return {
        Accept: "application/vnd.github.raw+json",
    };
}

function githubJsonHeaders() {
    return {
        Accept: "application/vnd.github+json",
    };
}

async function fetchWithRateLimitMessage(fetchImpl, url, options = {}) {
    const response = await fetchImpl(url, options);
    if (response.ok) {
        return response;
    }

    const remaining = response.headers.get("x-ratelimit-remaining");
    if (response.status === 403 && remaining === "0") {
        throw new Error("GitHub API rate limit reached for unauthenticated browser fetches");
    }

    throw new Error(`GitHub request failed: ${response.status} ${response.statusText}`);
}

async function fetchRepositoryTree(fetchImpl, owner, repo, ref) {
    const url =
        `https://api.github.com/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}` +
        `/git/trees/${encodeURIComponent(ref)}?recursive=1`;
    const response = await fetchWithRateLimitMessage(fetchImpl, url, {
        headers: githubJsonHeaders(),
    });
    return response.json();
}

async function fetchFileBytes(fetchImpl, owner, repo, ref, path) {
    const encodedPath = normalizePath(path)
        .split("/")
        .map((segment) => encodeURIComponent(segment))
        .join("/");
    const url =
        `https://api.github.com/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}` +
        `/contents/${encodedPath}?ref=${encodeURIComponent(ref)}`;
    const response = await fetchWithRateLimitMessage(fetchImpl, url, {
        headers: rawContentHeaders(),
    });
    return new Uint8Array(await response.arrayBuffer());
}

export function parseGitHubRepo(value) {
    if (typeof value !== "string") {
        throw new Error("GitHub repository must be a string like owner/repo");
    }

    let normalized = value.trim();
    if (!normalized) {
        throw new Error("GitHub repository must not be empty");
    }

    normalized = normalized
        .replace(/^https?:\/\/github\.com\//i, "")
        .replace(/\.git$/i, "")
        .replace(/\/+$/g, "");

    const segments = normalized.split("/").filter(Boolean);
    if (segments.length !== 2) {
        throw new Error("GitHub repository must look like owner/repo");
    }

    return {
        owner: segments[0],
        repo: segments[1],
    };
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

        if (selected.length >= limits.maxFiles) {
            stats.skippedFileLimit += 1;
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
    const cacheKey = buildCacheKey({ owner, repo, ref, limits });

    if (repoCache.has(cacheKey)) {
        return repoCache.get(cacheKey);
    }

    const loadPromise = (async () => {
        const tree = await fetchRepositoryTree(fetchImpl, owner, repo, ref);
        const selection = selectGitHubTreeEntries(tree.tree, limits);
        const inputs = [];
        let bytesRead = 0;
        let skippedBinaryContent = 0;
        let skippedBudget = 0;

        for (const entry of selection.selected) {
            const bytes = await fetchFileBytes(fetchImpl, owner, repo, ref, entry.path);

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
        }

        if (inputs.length === 0) {
            throw new Error("No browser-safe text files remained after GitHub filtering and limits");
        }

        return {
            inputs,
            source: {
                repo: `${owner}/${repo}`,
                ref,
                strategy: "github-tree-contents",
            },
            ingest: {
                bytesRead,
                loadedFiles: inputs.length,
                skippedBinaryContent,
                skippedBudget,
                ...selection.stats,
                ...limits,
            },
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
