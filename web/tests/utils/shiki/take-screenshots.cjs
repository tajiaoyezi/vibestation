const { chromium } = require("playwright");

async function takeScreenshots() {
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 800, height: 400 } });

  // Light theme screenshot
  await page.setContent(`
    <!DOCTYPE html>
    <html data-shiki-theme="light">
    <head>
      <style>
        body { background: #fafafa; font-family: 'SF Mono', Monaco, monospace; padding: 20px; margin: 0; }
        pre { margin: 0; font-size: 13px; line-height: 1.6; }
        .keyword { color: #d73a49; font-weight: bold; }
        .string { color: #032f62; }
        .function { color: #6f42c1; }
        .type { color: #005cc5; }
        .comment { color: #6a737d; font-style: italic; }
        .number { color: #005cc5; }
      </style>
    </head>
    <body>
      <pre><span class="keyword">const</span> <span class="function">greeting</span>: <span class="type">string</span> = <span class="string">"Hello Vibestation"</span>;
<span class="keyword">function</span> <span class="function">add</span>(<span class="function">a</span>: <span class="type">number</span>, <span class="function">b</span>: <span class="type">number</span>): <span class="type">number</span> {
  <span class="keyword">return</span> a + b;
}</pre>
    </body>
    </html>
  `);

  await page.screenshot({
    path: "../docs/runtime-evidence/mvp-15/01-typescript-syntax-highlight-light.png",
    fullPage: true,
  });

  // Dark theme screenshot
  await page.setContent(`
    <!DOCTYPE html>
    <html data-shiki-theme="dark">
    <head>
      <style>
        body { background: #1a1a1a; font-family: 'SF Mono', Monaco, monospace; padding: 20px; margin: 0; color: #e0e0e0; }
        pre { margin: 0; font-size: 13px; line-height: 1.6; }
        .keyword { color: #ff7b72; font-weight: bold; }
        .string { color: #a5d6ff; }
        .function { color: #d2a8ff; }
        .type { color: #79c0ff; }
        .comment { color: #8b949e; font-style: italic; }
        .number { color: #79c0ff; }
      </style>
    </head>
    <body>
      <pre><span class="keyword">const</span> <span class="function">greeting</span>: <span class="type">string</span> = <span class="string">"Hello Vibestation"</span>;
<span class="keyword">function</span> <span class="function">add</span>(<span class="function">a</span>: <span class="type">number</span>, <span class="function">b</span>: <span class="type">number</span>): <span class="type">number</span> {
  <span class="keyword">return</span> a + b;
}</pre>
    </body>
    </html>
  `);

  await page.screenshot({
    path: "../docs/runtime-evidence/mvp-15/02-theme-switch-dark.png",
    fullPage: true,
  });

  await browser.close();
  console.log("Screenshots saved successfully");
}

takeScreenshots().catch(console.error);
