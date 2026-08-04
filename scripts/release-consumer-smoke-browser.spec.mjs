import { test, expect } from "@playwright/test";
import { readFileSync } from "node:fs";

const baseUrl = process.env.BASE_URL;
const zipPath = process.env.ZIP_PATH;
const expectedVersion = process.env.EXPECTED_VERSION;
const consoleErrorsByPage = new WeakMap();

test.setTimeout(300_000);

test.beforeEach(async ({ page }) => {
    const consoleErrors = [];
    consoleErrorsByPage.set(page, consoleErrors);
    page.on("console", (message) => {
        if (message.type() === "error") {
            consoleErrors.push(message.text());
        }
    });
    page.on("pageerror", (error) => consoleErrors.push(error.message));
});

test.afterEach(async ({ page }, testInfo) => {
    const consoleErrors = consoleErrorsByPage.get(page) ?? [];
    if (consoleErrors.length > 0) {
        await testInfo.attach("browser-console-errors", {
            body: consoleErrors.join("\n"),
            contentType: "text/plain",
        });
    }
    expect(consoleErrors).toEqual([]);
});

async function runMode(page, mode, args = null) {
    const modeInput = page.locator("[data-mode]");
    const resultOutput = page.locator("[data-result]");
    await modeInput.selectOption(mode);
    expect(await modeInput.inputValue()).toBe(mode);
    if (args !== null) {
        await page.locator("[data-args]").fill(JSON.stringify(args));
    }
    const previous = await resultOutput.textContent();
    await resultOutput.evaluate((element) => {
        element.textContent = "";
    });
    await page.locator("[data-run]").click();
    await expect
        .poll(async () => {
            const value = await resultOutput.textContent();
            return value && value !== previous ? value : "";
        }, { timeout: 30_000 })
        .not.toBe("");
    const text = await resultOutput.textContent();
    const result = JSON.parse(text);
    expect(result).toBeTruthy();
    expect(result.data ?? result).toBeTruthy();
    return result;
}

test("released archive-enabled WASM supports browser ZIP workflows", async ({ page }) => {
    if (!baseUrl || !zipPath || !expectedVersion) {
        throw new Error("BASE_URL, ZIP_PATH, and EXPECTED_VERSION are required");
    }

    await page.goto(baseUrl, { waitUntil: "networkidle" });
    await expect(page.locator("[data-worker-capabilities]")).toContainText(expectedVersion);

    await page.locator("[data-zip-archive]").setInputFiles(zipPath);
    await expect(page.locator("[data-load-zip]")).toBeEnabled();
    await page.locator("[data-load-zip]").click();
    await expect(page.locator("[data-load-status]")).toContainText(/loaded/i, { timeout: 30_000 });

    for (const mode of ["lang", "module", "export"]) {
        await runMode(page, mode);
    }

    await runMode(page, "analyze", { preset: "receipt" });

    const downloadPromise = page.waitForEvent("download");
    await page.locator("[data-download]").click();
    const download = await downloadPromise;
    expect(await download.failure()).toBeNull();
    const downloadPath = await download.path();
    expect(downloadPath).toBeTruthy();
    const downloadedBytes = readFileSync(downloadPath);
    expect(downloadedBytes.byteLength).toBeGreaterThan(0);
    const filename = download.suggestedFilename().toLowerCase();
    if (filename.endsWith(".json")) {
        const downloaded = JSON.parse(downloadedBytes.toString("utf-8"));
        expect(downloaded).toBeTruthy();
    } else if (filename.endsWith(".zip")) {
        expect(downloadedBytes.subarray(0, 2).toString("ascii")).toBe("PK");
    } else {
        expect(downloadedBytes.toString("utf-8").trim()).not.toBe("");
    }
});
