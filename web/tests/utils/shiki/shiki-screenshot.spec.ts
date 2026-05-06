import { test, expect } from "@playwright/test";
import { createShikiAdapter } from "../../../src/utils/shiki";

test("TypeScript syntax highlight in light theme", async ({ page }) => {
  const adapter = createShikiAdapter();
  const code = `const greeting: string = "Hello Vibestation";
function add(a: number, b: number): number {
  return a + b;
}`;
  const html = await adapter.highlight(code, "typescript", "light");

  await page.setContent(`
    <!DOCTYPE html>
    <html data-shiki-theme="light">
    <head>
      <style>
        body { background: #fafafa; font-family: monospace; padding: 20px; }
        .shiki { font-size: 14px; line-height: 1.6; }
      </style>
    </head>
    <body>
      <pre class="shiki">${html}</pre>
    </body>
    </html>
  `);

  await page.screenshot({
    path: "../docs/runtime-evidence/mvp-15/01-typescript-syntax-highlight-light.png",
    fullPage: true,
  });

  const content = await page.textContent(".shiki");
  expect(content).toContain("const");
  expect(content).toContain("function");
});

test("TypeScript syntax highlight in dark theme", async ({ page }) => {
  const adapter = createShikiAdapter();
  const code = `const greeting: string = "Hello Vibestation";
function add(a: number, b: number): number {
  return a + b;
}`;
  const html = await adapter.highlight(code, "typescript", "dark");

  await page.setContent(`
    <!DOCTYPE html>
    <html data-shiki-theme="dark">
    <head>
      <style>
        body { background: #1a1a1a; font-family: monospace; padding: 20px; color: #e0e0e0; }
        .shiki { font-size: 14px; line-height: 1.6; }
      </style>
    </head>
    <body>
      <pre class="shiki">${html}</pre>
    </body>
    </html>
  `);

  await page.screenshot({
    path: "../docs/runtime-evidence/mvp-15/02-theme-switch-dark.png",
    fullPage: true,
  });

  const content = await page.textContent(".shiki");
  expect(content).toContain("const");
  expect(content).toContain("function");
});
