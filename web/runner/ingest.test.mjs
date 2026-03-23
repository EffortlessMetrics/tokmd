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

test("parseGitHubRepo accepts owner/repo and GitHub URLs", () => {
    assert.deepEqual(parseGitHubRepo("EffortlessMetrics/tokmd"), {
        owner: "EffortlessMetrics",
        repo: "tokmd",
    });
    assert.deepEqual(parseGitHubRepo("https://github.com/EffortlessMetrics/tokmd"), {
        owner: "EffortlessMetrics",
        repo: "tokmd",
    });
    assert.throws(() => parseGitHubRepo("tokmd"), /owner\/repo/);
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
        ["README.md", "src/lib.rs"]
    );
    assert.equal(result.stats.skippedVendor, 1);
    assert.equal(result.stats.skippedBinaryPath, 1);
    assert.equal(result.stats.skippedTooLarge, 1);
});

test("fetchGitHubRepoInputs materializes ordered inputs and reuses the in-memory cache", async () => {
    clearGitHubRepoCache();

    const calls = [];
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
    });

    assert.deepEqual(
        first.inputs.map((entry) => entry.path),
        ["README.md", "src/lib.rs"]
    );
    assert.equal(first.ingest.loadedFiles, 2);
    assert.equal(first.ingest.skippedVendor, 1);
    assert.equal(first.source.strategy, "github-tree-contents");
    assert.equal(calls.length, 3);
    assert.equal(second, first);
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
