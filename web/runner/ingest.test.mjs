import assert from "node:assert/strict";
import test from "node:test";

import {
    clearGitHubRepoCache,
    fetchGitHubRepoInputs,
    parseGitHubRepo,
    selectGitHubTreeEntries,
} from "./ingest.js";

function jsonResponse(value, headers = {}) {
    return new Response(JSON.stringify(value), {
        status: 200,
        headers: {
            "content-type": "application/json",
            ...headers,
        },
    });
}

function textResponse(value, headers = {}) {
    return new Response(value, {
        status: 200,
        headers: {
            "content-type": "text/plain; charset=utf-8",
            ...headers,
        },
    });
}

function deferred() {
    let resolve;
    let reject;
    const promise = new Promise((resolvePromise, rejectPromise) => {
        resolve = resolvePromise;
        reject = rejectPromise;
    });
    return { promise, resolve, reject };
}

test("parseGitHubRepo accepts owner/repo and GitHub URLs", () => {
    assert.deepEqual(parseGitHubRepo("EffortlessMetrics/tokmd"), {
        owner: "EffortlessMetrics",
        repo: "tokmd",
    });
    assert.deepEqual(parseGitHubRepo("https://github.com/EffortlessMetrics/tokmd"), {
        owner: "EffortlessMetrics",
        repo: "tokmd",
    });
    assert.deepEqual(parseGitHubRepo("https://github.com/EffortlessMetrics/tokmd?tab=readme"), {
        owner: "EffortlessMetrics",
        repo: "tokmd",
    });
    assert.deepEqual(parseGitHubRepo("https://github.com/EffortlessMetrics/tokmd.git/"), {
        owner: "EffortlessMetrics",
        repo: "tokmd",
    });
    assert.throws(() => parseGitHubRepo("tokmd"), /owner\/repo/);
    assert.throws(() => parseGitHubRepo("git@github.com:EffortlessMetrics/tokmd.git"), /owner\/repo/);
});

test("selectGitHubTreeEntries filters vendor, binary, and oversized files deterministically", () => {
    const result = selectGitHubTreeEntries(
        [
            { path: "_fix.py", size: 10, type: "blob" },
            { path: "vendor/lib.js", size: 20, type: "blob" },
            { path: "src/logo.png", size: 20, type: "blob" },
            { path: "README.md", size: 64, type: "blob" },
            { path: "src/lib.rs", size: 90, type: "blob" },
            { path: "src/huge.rs", size: 9000, type: "blob" },
        ],
        { maxFiles: 2, maxFileBytes: 512 }
    );

    assert.deepEqual(
        result.selected.map((entry) => entry.path),
        ["README.md", "src/lib.rs", "_fix.py"]
    );
    assert.equal(result.stats.skippedVendor, 1);
    assert.equal(result.stats.skippedBinaryPath, 1);
    assert.equal(result.stats.skippedTooLarge, 1);
    assert.equal(result.stats.skippedFileLimit, 0);
});

test("fetchGitHubRepoInputs materializes ordered inputs and reuses the in-memory cache", async () => {
    clearGitHubRepoCache();

    const calls = [];
    const secondProgress = [];
    const fetchImpl = async (url, options = {}) => {
        calls.push({ url, accept: options.headers?.Accept ?? null });

        if (url.includes("/git/trees/")) {
            return jsonResponse({
                tree: [
                    { path: "vendor/lib.js", size: 20, type: "blob" },
                    { path: "README.md", size: 32, type: "blob" },
                    { path: "src/lib.rs", size: 48, type: "blob" },
                ],
            });
        }

        if (url.includes("/contents/README.md")) {
            return textResponse("# tokmd\n");
        }

        if (url.includes("/contents/src/lib.rs")) {
            return textResponse("pub fn alpha() {}\n");
        }

        throw new Error(`unexpected fetch url: ${url}`);
    };

    const first = await fetchGitHubRepoInputs({
        repo: "EffortlessMetrics/tokmd",
        ref: "main",
        fetchImpl,
        maxFiles: 8,
        maxBytes: 512,
        maxFileBytes: 256,
    });
    const second = await fetchGitHubRepoInputs({
        repo: "EffortlessMetrics/tokmd",
        ref: "main",
        fetchImpl,
        maxFiles: 8,
        maxBytes: 512,
        maxFileBytes: 256,
        onProgress: (update) => secondProgress.push(update.phase),
    });

    assert.deepEqual(
        first.inputs.map((entry) => entry.path),
        ["README.md", "src/lib.rs"]
    );
    assert.equal(first.ingest.loadedFiles, 2);
    assert.equal(first.ingest.skippedVendor, 1);
    assert.equal(first.source.strategy, "github-tree-contents");
    assert.equal(first.ingest.cache.hit, false);
    assert.equal(first.ingest.authMode, "anonymous");
    assert.equal(calls.length, 3);
    assert.notEqual(second, first);
    assert.equal(second.ingest.cache.hit, true);
    assert.deepEqual(secondProgress, ["cache", "complete"]);
});

test("fetchGitHubRepoInputs backfills after early fetch-time skips", async () => {
    clearGitHubRepoCache();

    const fetchImpl = async (url) => {
        if (url.includes("/git/trees/")) {
            return jsonResponse({
                tree: [
                    { path: "src/a.txt", size: 8, type: "blob" },
                    { path: "src/b.txt", size: 8, type: "blob" },
                    { path: "src/c.txt", size: 8, type: "blob" },
                ],
            });
        }

        if (url.includes("/contents/src/a.txt")) {
            return new Response(new Uint8Array([0, 1, 2]).buffer, {
                status: 200,
                headers: { "content-type": "application/octet-stream" },
            });
        }

        if (url.includes("/contents/src/b.txt")) {
            return textResponse("alpha\n");
        }

        if (url.includes("/contents/src/c.txt")) {
            return textResponse("beta\n");
        }

        throw new Error(`unexpected fetch url: ${url}`);
    };

    const result = await fetchGitHubRepoInputs({
        repo: "EffortlessMetrics/tokmd",
        ref: "main",
        fetchImpl,
        maxFiles: 2,
        maxBytes: 128,
        maxFileBytes: 64,
    });

    assert.deepEqual(
        result.inputs.map((entry) => entry.path),
        ["src/b.txt", "src/c.txt"]
    );
    assert.equal(result.ingest.loadedFiles, 2);
    assert.equal(result.ingest.skippedBinaryContent, 1);
});

test("fetchGitHubRepoInputs marks truncated tree listings as partial", async () => {
    clearGitHubRepoCache();

    const fetchImpl = async (url) => {
        if (url.includes("/git/trees/")) {
            return jsonResponse({
                truncated: true,
                tree: [{ path: "src/lib.rs", size: 32, type: "blob" }],
            });
        }

        if (url.includes("/contents/src/lib.rs")) {
            return textResponse("pub fn alpha() {}\n");
        }

        throw new Error(`unexpected fetch url: ${url}`);
    };

    const result = await fetchGitHubRepoInputs({
        repo: "EffortlessMetrics/tokmd",
        ref: "main",
        fetchImpl,
    });

    assert.equal(result.ingest.partial, true);
    assert.equal(result.ingest.treeEntriesTruncated, true);
    assert.deepEqual(
        result.ingest.partialReasons.map((reason) => reason.code),
        ["tree_truncated"]
    );
});

test("fetchGitHubRepoInputs fails cleanly when nothing browser-safe remains", async () => {
    clearGitHubRepoCache();

    const fetchImpl = async (url) => {
        if (url.includes("/git/trees/")) {
            return jsonResponse({
                tree: [{ path: "vendor/lib.js", size: 20, type: "blob" }],
            });
        }

        throw new Error(`unexpected fetch url: ${url}`);
    };

    await assert.rejects(
        () =>
            fetchGitHubRepoInputs({
                repo: "EffortlessMetrics/tokmd",
                ref: "main",
                fetchImpl,
            }),
        /No browser-safe text files/
    );
});

test("fetchGitHubRepoInputs forwards token auth and tracks auth mode", async () => {
    clearGitHubRepoCache();

    const calls = [];
    const fetchImpl = async (url, options = {}) => {
        calls.push({
            url,
            authorization: options.headers?.Authorization ?? null,
        });

        if (url.includes("/git/trees/")) {
            return jsonResponse({
                tree: [{ path: "README.md", size: 32, type: "blob" }],
            });
        }

        if (url.includes("/contents/README.md")) {
            return textResponse("# tokmd\n");
        }

        throw new Error(`unexpected fetch url: ${url}`);
    };

    const result = await fetchGitHubRepoInputs({
        repo: "EffortlessMetrics/tokmd",
        ref: "main",
        token: "secret-token",
        fetchImpl,
    });

    assert.equal(result.ingest.authMode, "token");
    assert.deepEqual(
        calls.map((call) => call.authorization),
        ["token secret-token", "token secret-token"]
    );
});

test("fetchGitHubRepoInputs surfaces auth and private-repo access errors", async () => {
    clearGitHubRepoCache();

    await assert.rejects(
        () =>
            fetchGitHubRepoInputs({
                repo: "EffortlessMetrics/tokmd",
                ref: "main",
                fetchImpl: async () =>
                    new Response(JSON.stringify({ message: "Bad credentials" }), {
                        status: 401,
                        headers: { "content-type": "application/json" },
                    }),
            }),
        (error) => error?.code === "github_auth_required"
    );

    await assert.rejects(
        () =>
            fetchGitHubRepoInputs({
                repo: "EffortlessMetrics/tokmd",
                ref: "main",
                fetchImpl: async () =>
                    new Response(JSON.stringify({ message: "Not Found" }), {
                        status: 404,
                        headers: { "content-type": "application/json" },
                    }),
            }),
        (error) => error?.code === "github_repo_unavailable"
    );
});

test("fetchGitHubRepoInputs surfaces primary and secondary rate limits", async () => {
    clearGitHubRepoCache();

    await assert.rejects(
        () =>
            fetchGitHubRepoInputs({
                repo: "EffortlessMetrics/tokmd",
                ref: "main",
                fetchImpl: async () =>
                    new Response(JSON.stringify({ message: "API rate limit exceeded" }), {
                        status: 403,
                        headers: {
                            "content-type": "application/json",
                            "x-ratelimit-remaining": "0",
                            "x-ratelimit-reset": "2000000000",
                        },
                    }),
            }),
        (error) =>
            error?.code === "github_primary_rate_limit" &&
            error?.resetAt === "2033-05-18T03:33:20.000Z"
    );

    await assert.rejects(
        () =>
            fetchGitHubRepoInputs({
                repo: "EffortlessMetrics/tokmd",
                ref: "main",
                fetchImpl: async () =>
                    new Response(JSON.stringify({ message: "You have exceeded a secondary rate limit." }), {
                        status: 429,
                        headers: {
                            "content-type": "application/json",
                            "retry-after": "12",
                        },
                    }),
            }),
        (error) =>
            error?.code === "github_secondary_rate_limit" &&
            error?.retryAfterSeconds === 12
    );
});

test("fetchGitHubRepoInputs honors AbortSignal before starting network work", async () => {
    clearGitHubRepoCache();

    const controller = new AbortController();
    controller.abort();

    await assert.rejects(
        () =>
            fetchGitHubRepoInputs({
                repo: "EffortlessMetrics/tokmd",
                ref: "main",
                signal: controller.signal,
                fetchImpl: async () => {
                    throw new Error("fetch should not run after abort");
                },
            }),
        (error) => error?.name === "AbortError" && error?.code === "repo_load_aborted"
    );
});

test("fetchGitHubRepoInputs normalizes mid-flight aborts", async () => {
    clearGitHubRepoCache();

    const controller = new AbortController();
    const pending = fetchGitHubRepoInputs({
        repo: "EffortlessMetrics/tokmd",
        ref: "main",
        signal: controller.signal,
        fetchImpl: async (_url, options = {}) =>
            await new Promise((_, reject) => {
                options.signal.addEventListener(
                    "abort",
                    () => reject(Object.assign(new Error("aborted"), { name: "AbortError" })),
                    { once: true }
                );
            }),
    });

    controller.abort();

    await assert.rejects(
        () => pending,
        (error) => error?.name === "AbortError" && error?.code === "repo_load_aborted"
    );
});

test("fetchGitHubRepoInputs lets cache-hit waiters abort independently", async () => {
    clearGitHubRepoCache();

    const fileGate = deferred();
    const controller = new AbortController();
    const fetchImpl = async (url) => {
        if (url.includes("/git/trees/")) {
            return jsonResponse({
                tree: [{ path: "README.md", size: 32, type: "blob" }],
            });
        }

        if (url.includes("/contents/README.md")) {
            await fileGate.promise;
            return textResponse("# tokmd\n");
        }

        throw new Error(`unexpected fetch url: ${url}`);
    };

    const coldLoad = fetchGitHubRepoInputs({
        repo: "EffortlessMetrics/tokmd",
        ref: "main",
        fetchImpl,
    });
    const cachedWaiter = fetchGitHubRepoInputs({
        repo: "EffortlessMetrics/tokmd",
        ref: "main",
        fetchImpl,
        signal: controller.signal,
    });

    controller.abort();
    fileGate.resolve();

    const coldResult = await coldLoad;
    assert.equal(coldResult.ingest.loadedFiles, 1);
    await assert.rejects(
        () => cachedWaiter,
        (error) => error?.name === "AbortError" && error?.code === "repo_load_aborted"
    );
});

test("fetchGitHubRepoInputs rejects non-positive limits", async () => {
    clearGitHubRepoCache();

    await assert.rejects(
        () =>
            fetchGitHubRepoInputs({
                repo: "EffortlessMetrics/tokmd",
                ref: "main",
                maxFiles: 0,
                fetchImpl: async () => {
                    throw new Error("fetch should not run for invalid limits");
                },
            }),
        /maxFiles must be a positive number/
    );
});

test("fetchGitHubRepoInputs emits progress and partial-load markers", async () => {
    clearGitHubRepoCache();

    const progress = [];
    const fetchImpl = async (url) => {
        if (url.includes("/git/trees/")) {
            return jsonResponse({
                tree: [
                    { path: "README.md", size: 8, type: "blob" },
                    { path: "src/huge.rs", size: 1000, type: "blob" },
                    { path: "src/lib.rs", size: 64, type: "blob" },
                    { path: "src/extra.rs", size: 32, type: "blob" },
                ],
            });
        }

        if (url.includes("/contents/README.md")) {
            return textResponse("# tokmd\n");
        }

        if (url.includes("/contents/src/lib.rs")) {
            return textResponse("pub fn alpha() {}\n");
        }

        if (url.includes("/contents/src/extra.rs")) {
            return textResponse("pub fn beta() {}\n");
        }

        throw new Error(`unexpected fetch url: ${url}`);
    };

    const result = await fetchGitHubRepoInputs({
        repo: "EffortlessMetrics/tokmd",
        ref: "main",
        fetchImpl,
        maxFiles: 4,
        maxBytes: 32,
        maxFileBytes: 128,
        onProgress: (update) => progress.push(update),
    });

    assert.equal(progress[0].phase, "tree");
    assert.equal(progress.at(-1).phase, "complete");
    assert.equal(result.ingest.partial, true);
    assert.deepEqual(
        result.ingest.partialReasons.map((reason) => reason.code),
        ["too_large_files", "byte_budget"]
    );
});
