export const GITHUB_TOKEN_STORAGE_KEY = "tokmd.githubToken";

function normalizeToken(value) {
    return typeof value === "string" ? value.trim() : "";
}

export function resolveSessionStorage(source = globalThis) {
    try {
        return source?.sessionStorage ?? null;
    } catch {
        return null;
    }
}

export function readSessionToken(storage) {
    if (!storage || typeof storage.getItem !== "function") {
        return "";
    }

    try {
        return normalizeToken(storage.getItem(GITHUB_TOKEN_STORAGE_KEY));
    } catch {
        return "";
    }
}

export function writeSessionToken(storage, value) {
    const token = normalizeToken(value);

    if (!storage) {
        return token;
    }

    try {
        if (token && typeof storage.setItem === "function") {
            storage.setItem(GITHUB_TOKEN_STORAGE_KEY, token);
        } else if (typeof storage.removeItem === "function") {
            storage.removeItem(GITHUB_TOKEN_STORAGE_KEY);
        }
    } catch {
        // Browsers can block sessionStorage; keep the in-memory field usable.
    }

    return token;
}

export function clearSessionToken(storage) {
    if (!storage || typeof storage.removeItem !== "function") {
        return;
    }

    try {
        storage.removeItem(GITHUB_TOKEN_STORAGE_KEY);
    } catch {
        // Browser storage failures should not block clearing the visible field.
    }
}

export function authModeForToken(value) {
    return normalizeToken(value) ? "authenticated" : "anonymous";
}
