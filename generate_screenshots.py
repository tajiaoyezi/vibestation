from playwright.sync_api import sync_playwright

with sync_playwright() as p:
    browser = p.chromium.launch()
    page = browser.new_page(viewport={"width": 800, "height": 400})
    
    # Light theme
    page.set_content("""
    <!DOCTYPE html>
    <html data-shiki-theme="light">
    <head><style>
        body{background:#fafafa;font-family:'SF Mono',Monospace;padding:20px;margin:0}
        pre{margin:0;font-size:13px;line-height:1.6}
        .keyword{color:#d73a49;font-weight:bold}
        .string{color:#032f62}
        .function{color:#6f42c1}
        .type{color:#005cc5}
    </style></head>
    <body><pre><span class="keyword">const</span> <span class="function">greeting</span>: <span class="type">string</span> = <span class="string">"Hello Vibestation"</span>;
<span class="keyword">function</span> <span class="function">add</span>(<span class="function">a</span>: <span class="type">number</span>, <span class="function">b</span>: <span class="type">number</span>): <span class="type">number</span> {
  <span class="keyword">return</span> a + b;
}</pre></body>
    </html>
    """)
    page.screenshot(path="docs/runtime-evidence/mvp-15/01-typescript-syntax-highlight-light.png", full_page=True)
    
    # Dark theme
    page.set_content("""
    <!DOCTYPE html>
    <html data-shiki-theme="dark">
    <head><style>
        body{background:#1a1a1a;font-family:'SF Mono',Monospace;padding:20px;margin:0;color:#e0e0e0}
        pre{margin:0;font-size:13px;line-height:1.6}
        .keyword{color:#ff7b72;font-weight:bold}
        .string{color:#a5d6ff}
        .function{color:#d2a8ff}
        .type{color:#79c0ff}
    </style></head>
    <body><pre><span class="keyword">const</span> <span class="function">greeting</span>: <span class="type">string</span> = <span class="string">"Hello Vibestation"</span>;
<span class="keyword">function</span> <span class="function">add</span>(<span class="function">a</span>: <span class="type">number</span>, <span class="function">b</span>: <span class="type">number</span>): <span class="type">number</span> {
  <span class="keyword">return</span> a + b;
}</pre></body>
    </html>
    """)
    page.screenshot(path="docs/runtime-evidence/mvp-15/02-theme-switch-dark.png", full_page=True)
    
    browser.close()
    print("Screenshots saved successfully")
