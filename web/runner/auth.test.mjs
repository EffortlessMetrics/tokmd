import test from "node:test";
import assert from "node:assert/strict";

import {
    GITHUB_TOKEN_STORAGE_KEY,
    authModeForToken,
    clearSessionToken,
    readSessionToken,
    resolveSessionStorage,
    writeSessionToken,
} from "./auth.js";

function createMemoryStorage() {
    const values = new Map();

    return {
        getItem(key) {
            return values.has(key) ? values.get(key) : null;
        },
        removeItem(key) {
            values.delete(key);
        },
        setItem(key, value) {
            values.set(key, String(value));
        },
    };
}

test("session token helpers read, trim, write, and clear token values", () => {
    const storage = createMemoryStorage();

    assert.equal(readSessionToken(storage), "");
    assert.equal(writeSessionToken(storage, "  test-token-example  "), "test-token-example");
    assert.equal(storage.getItem(GITHUB_TOKEN_STORAGE_KEY), "test-token-example");
    assert.equal(readSessionToken(storage), "test-token-example");

    assert.equal(writeSessionToken(storage, "   "), "");
    assert.equal(storage.getItem(GITHUB_TOKEN_STORAGE_KEY), null);

    writeSessionToken(storage, "second");
    clearSessionToken(storage);
    assert.equal(readSessionToken(storage), "");
});

test("session token helpers tolerate unavailable storage", () => {
    const throwingStorage = {
        getItem() {
            throw new Error("blocked");
        },
        removeItem() {
            throw new Error("blocked");
        },
        setItem() {
            throw new Error("blocked");
        },
    };

    assert.equal(readSessionToken(throwingStorage), "");
    assert.equal(writeSessionToken(throwingStorage, " token "), "token");
    assert.doesNotThrow(() => clearSessionToken(throwingStorage));

    assert.equal(resolveSessionStorage({}), null);
    assert.equal(
        resolveSessionStorage({
            get sessionStorage() {
                throw new Error("blocked");
            },
        }),
        null,
    );
});

test("auth mode never exposes token text", () => {
    assert.equal(authModeForToken(""), "anonymous");
    assert.equal(authModeForToken("   "), "anonymous");
    assert.equal(authModeForToken("test-token-value"), "authenticated");
});
