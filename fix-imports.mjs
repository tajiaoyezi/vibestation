// 修正 unify-platform 脚本插入错误的 import
import { readFileSync, writeFileSync } from "node:fs";

// Terminal.tsx: 删错误位置的 import，加到正确位置（hooks import 之前）
{
  const f = "web/src/panels/Terminal/Terminal.tsx";
  let s = readFileSync(f, "utf8");
  const nl = s.includes("\r\n") ? "\r\n" : "\n";
  // 删错误插入的行
  const badLine = `${nl}import { isMacPlatform } from "../lib/platform";`;
  s = s.replace(badLine, "");
  // 在 usePaneNavigation import 后加
  const anchor = 'import { usePaneMaximizeToggle, usePaneNavigation } from "./usePaneNavigation";';
  if (!s.includes(anchor)) { console.error(`FAIL ${f}: anchor not found`); process.exit(1); }
  s = s.replace(anchor, anchor + nl + 'import { isMacPlatform } from "../lib/platform";');
  writeFileSync(f, s, "utf8");
  console.log("OK Terminal.tsx");
}

// usePaneNavigation.ts: 删错误位置 import，加到正确位置
{
  const f = "web/src/panels/Terminal/usePaneNavigation.ts";
  let s = readFileSync(f, "utf8");
  const nl = s.includes("\r\n") ? "\r\n" : "\n";
  const badLine = `${nl}import { isMacPlatform } from "../../lib/platform";`;
  s = s.replace(badLine, "");
  // 找第一个 import 块结束，在最后 import 后加
  const lines = s.split(nl);
  let lastImportIdx = -1;
  for (let i = 0; i < lines.length; i++) {
    if (lines[i].match(/^import\s/) || (lastImportIdx >= 0 && lines[i].match(/^\s*(\}|\{|[A-Z])/) && i <= lastImportIdx + 5)) {
      // 简化：找以 } from 结尾的行作为 import 块结束
      if (lines[i].includes("from ")) lastImportIdx = i;
    } else if (lastImportIdx >= 0 && !lines[i].match(/^import/) && !lines[i].match(/^\s/) && lines[i].trim() !== "") {
      break;
    }
  }
  if (lastImportIdx < 0) { console.error(`FAIL ${f}: no import found`); process.exit(1); }
  lines.splice(lastImportIdx + 1, 0, 'import { isMacPlatform } from "../../lib/platform";');
  writeFileSync(f, lines.join(nl), "utf8");
  console.log("OK usePaneNavigation.ts");
}

// mvp17-keyboard.ts: 加缺失的 import
{
  const f = "web/src/lib/mvp17-keyboard.ts";
  let s = readFileSync(f, "utf8");
  if (!s.includes("isMacPlatform") || s.includes('import { isMacPlatform }')) { console.log("SKIP mvp17-keyboard.ts (already has or no usage)"); }
  else {
    const nl = s.includes("\r\n") ? "\r\n" : "\n";
    // 找最后 } from 行
    const lines = s.split(nl);
    let lastImportIdx = -1;
    for (let i = 0; i < lines.length; i++) {
      if (lines[i].includes("from ") && lines[i].match(/["']/)) lastImportIdx = i;
      else if (lastImportIdx >= 0 && lines[i].trim() !== "" && !lines[i].match(/^\s/) && !lines[i].includes("from ")) break;
    }
    if (lastImportIdx < 0) { console.error(`FAIL ${f}: no import found`); process.exit(1); }
    lines.splice(lastImportIdx + 1, 0, 'import { isMacPlatform } from "./platform";');
    writeFileSync(f, lines.join(nl), "utf8");
    console.log("OK mvp17-keyboard.ts");
  }
}

// pane-keyboard.ts: 加缺失的 import
{
  const f = "web/src/lib/pane-keyboard.ts";
  let s = readFileSync(f, "utf8");
  if (s.includes('import { isMacPlatform }')) { console.log("SKIP pane-keyboard.ts"); }
  else {
    const nl = s.includes("\r\n") ? "\r\n" : "\n";
    const lines = s.split(nl);
    let lastImportIdx = -1;
    for (let i = 0; i < lines.length; i++) {
      if (lines[i].includes("from ") && lines[i].match(/["']/)) lastImportIdx = i;
      else if (lastImportIdx >= 0 && lines[i].trim() !== "" && !lines[i].match(/^\s/) && !lines[i].includes("from ")) break;
    }
    if (lastImportIdx < 0) { console.error(`FAIL ${f}: no import found`); process.exit(1); }
    lines.splice(lastImportIdx + 1, 0, 'import { isMacPlatform } from "./platform";');
    writeFileSync(f, lines.join(nl), "utf8");
    console.log("OK pane-keyboard.ts");
  }
}

// index.tsx: 把 isMacPlatform 加到已有的 platform import
{
  const f = "web/src/index.tsx";
  let s = readFileSync(f, "utf8");
  const oldImp = 'import { applyPlatformClass } from "./lib/platform";';
  const newImp = 'import { applyPlatformClass, isMacPlatform } from "./lib/platform";';
  if (!s.includes(oldImp)) { console.error(`FAIL ${f}: existing platform import not found`); process.exit(1); }
  s = s.replace(oldImp, newImp);
  writeFileSync(f, s, "utf8");
  console.log("OK index.tsx");
}

console.log("DONE: all imports fixed");
