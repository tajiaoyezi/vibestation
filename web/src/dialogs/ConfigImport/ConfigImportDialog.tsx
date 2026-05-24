import {
  createSignal,
  createMemo,
  onMount,
  For,
  Show,
  Switch,
  Match,
  type Component,
} from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type {
  AppliedField,
  ImportApplyRequest,
  ImportApplyResult,
  ImportFieldType,
  ImportPreview,
  ImportScanResult,
  ImportSource,
  KeyBindingConflict,
  KeyBindingResolution,
} from "../../bindings";
import { isFieldDisabled, getDisabledReason } from "./fieldRules";

import "./styles.css";

interface ConfigImportDialogProps {
  /** 用户关闭对话框（取消 / 完成） */
  onClose: () => void;
  /** apply 成功回调（settings store 已通过 settings_changed event 自动 reload） */
  onApplied?: (result: ImportApplyResult) => void;
}

type Step = "select" | "preview" | "confirm" | "result";

const SOURCE_LABELS: Record<ImportSource, string> = {
  ghostty: "Ghostty",
  iTerm2: "iTerm2",
  alacritty: "Alacritty",
};

const SOURCE_PLATFORMS: Record<ImportSource, string> = {
  ghostty: "macOS · Linux",
  iTerm2: "macOS",
  alacritty: "Linux",
};

interface FontFallback {
  font: string;
  available: boolean;
}

/**
 * MVP-06 Phase B · 配置导入对话框（Calm Studio 模态 · 3 step）
 *
 * 步骤：
 *   1. select   · 选择源（自动扫描结果展示 · path_exists / errors）
 *   2. preview  · 字段勾选 + 冲突 warning
 *   3. confirm  · 确认 + 字体 fallback 提示
 *   4. result   · 应用结果（applied / skipped / errors）
 */
export const ConfigImportDialog: Component<ConfigImportDialogProps> = (
  props,
) => {
  const [step, setStep] = createSignal<Step>("select");

  const [scanResults, setScanResults] = createSignal<ImportScanResult[]>([]);
  const [scanning, setScanning] = createSignal(true);
  const [scanError, setScanError] = createSignal<string | null>(null);
  // review round 5 fix: 拆分 scan / preview 错误状态 · 防 chooseSource preview 失败时
  // step 仍 "select" 但 selectedSource 已 set 的状态不一致 · UI 也无法区分两阶段错误
  const [previewError, setPreviewError] = createSignal<string | null>(null);

  const [selectedSource, setSelectedSource] = createSignal<ImportSource | null>(
    null,
  );
  const [preview, setPreview] = createSignal<ImportPreview | null>(null);
  const [checkedFields, setCheckedFields] = createSignal<Set<number>>(
    new Set(),
  );
  const [conflicts, setConflicts] = createSignal<KeyBindingConflict[]>([]);
  const [fontFallback, setFontFallback] = createSignal<FontFallback[]>([]);

  const [applying, setApplying] = createSignal(false);
  const [applyResult, setApplyResult] = createSignal<ImportApplyResult | null>(
    null,
  );

  onMount(async () => {
    await runScan();
  });

  async function runScan(): Promise<void> {
    setScanning(true);
    setScanError(null);
    try {
      const results = await invoke<ImportScanResult[]>("config_import_scan");
      setScanResults(results);
    } catch (err) {
      setScanError(err instanceof Error ? err.message : String(err));
    } finally {
      setScanning(false);
    }
  }

  async function chooseSource(source: ImportSource): Promise<void> {
    setSelectedSource(source);
    setPreviewError(null); // 清旧错误 · 进入新 preview
    try {
      const p = await invoke<ImportPreview>("config_import_preview", {
        selectedSources: [source],
      });
      setPreview(p);
      setConflicts(p.conflicts);

      // 默认勾选所有可用字段（用户可逐项取消 · spec §A.3）
      // Issue #205 Finding 2 (a) 修复：disabled 字段（v0.1 AnsiColor）不默认勾选
      const target = p.sources.find((s) => s.source === source);
      const allIdx = new Set<number>();
      target?.detectedFields.forEach((field, idx) => {
        if (!isFieldDisabled(field)) {
          allIdx.add(idx);
        }
      });
      setCheckedFields(allIdx);

      setStep("preview");
    } catch (err) {
      // review round 5 fix: preview 失败用独立 previewError signal（不污染 scanError）
      // 防 step 留 "select" 但 selectedSource 已 set 的状态不一致
      setPreviewError(err instanceof Error ? err.message : String(err));
      setSelectedSource(null); // rollback 选源 · 让用户重新选
    }
  }

  function skipManual(): void {
    // 跳过自动扫描 · 关掉对话框（用户后续可在 Settings 里手动配置）
    props.onClose();
  }

  function toggleField(idx: number): void {
    setCheckedFields((prev) => {
      const next = new Set(prev);
      if (next.has(idx)) {
        next.delete(idx);
      } else {
        next.add(idx);
      }
      return next;
    });
  }

  const currentSourceResult = createMemo<ImportScanResult | null>(() => {
    const src = selectedSource();
    if (!src) return null;
    return preview()?.sources.find((s) => s.source === src) ?? null;
  });

  function selectedFields(): ImportFieldType[] {
    const result = currentSourceResult();
    if (!result) return [];
    return result.detectedFields.filter((_, idx) => checkedFields().has(idx));
  }

  function isFieldConflicting(
    field: ImportFieldType,
  ): KeyBindingConflict | null {
    if (field.kind !== "keyBinding") return null;
    // Issue #206 Advisory #1 修复：仅按 sourceKey 匹配 conflict · 不能 OR sourceAction
    // · 否则不同 key 但同 action 的字段会被误标冲突（例 Cmd+T spawn_tab + Cmd+Shift+T
    // spawn_tab · 前者真冲突 · 后者会被误连坐）
    return conflicts().find((c) => c.sourceKey === field.key) ?? null;
  }

  function setConflictChoice(
    sourceKey: string,
    choice: KeyBindingResolution,
  ): void {
    setConflicts((prev) =>
      prev.map((c) =>
        c.sourceKey === sourceKey ? { ...c, userChoice: choice } : c,
      ),
    );
  }

  function checkFontAvailability(): FontFallback[] {
    const fields = selectedFields();
    return fields
      .filter(
        (f): f is { kind: "fontFamily"; value: string } =>
          f.kind === "fontFamily",
      )
      .map((f) => {
        const available =
          typeof document.fonts?.check === "function"
            ? safeFontCheck(f.value)
            : true;
        return { font: f.value, available };
      });
  }

  function gotoConfirm(): void {
    setFontFallback(checkFontAvailability());
    setStep("confirm");
  }

  async function applyImport(): Promise<void> {
    const src = selectedSource();
    if (!src || applying()) return;
    setApplying(true);
    try {
      // Issue #206 Advisory #2 修复：ImportApplyRequest.source 字段已删（apply 不引用）
      // · src 仍用于 step 状态判断 · 不传后端
      const req: ImportApplyRequest = {
        fields: selectedFields(),
        conflictResolutions: conflicts(),
      };
      const result = await invoke<ImportApplyResult>("config_import_apply", {
        req,
      });
      setApplyResult(result);
      props.onApplied?.(result);
      setStep("result");
    } catch (err) {
      setApplyResult({
        applied: [],
        skippedConflicts: [],
        errors: [err instanceof Error ? err.message : String(err)],
      });
      setStep("result");
    } finally {
      setApplying(false);
    }
  }

  function backToSelect(): void {
    setSelectedSource(null);
    setPreview(null);
    setConflicts([]);
    setCheckedFields(new Set<number>());
    setStep("select");
  }

  return (
    <div
      class="vs-config-import-overlay"
      role="dialog"
      aria-modal="true"
      aria-labelledby="vs-config-import-title"
    >
      <div class="vs-config-import-modal">
        <header class="vs-config-import-head">
          <h2 id="vs-config-import-title" class="vs-config-import-title">
            Import terminal config
          </h2>
          <button
            type="button"
            class="vs-config-import-close"
            aria-label="Close import dialog"
            onClick={props.onClose}
          >
            ×
          </button>
        </header>

        <div class="vs-config-import-stepper" aria-label="Import steps">
          <span
            classList={{
              "vs-config-import-step": true,
              active: step() === "select",
              done:
                step() === "preview" ||
                step() === "confirm" ||
                step() === "result",
            }}
          >
            1 · Source
          </span>
          <span class="vs-config-import-step-sep" />
          <span
            classList={{
              "vs-config-import-step": true,
              active: step() === "preview",
              done: step() === "confirm" || step() === "result",
            }}
          >
            2 · Preview
          </span>
          <span class="vs-config-import-step-sep" />
          <span
            classList={{
              "vs-config-import-step": true,
              active: step() === "confirm",
              done: step() === "result",
            }}
          >
            3 · Apply
          </span>
        </div>

        <div class="vs-config-import-body">
          <Switch>
            <Match when={step() === "select"}>
              <SelectStep
                scanning={scanning}
                scanError={scanError}
                previewError={previewError}
                scanResults={scanResults}
                onChoose={chooseSource}
                onSkip={skipManual}
                onRescan={runScan}
              />
            </Match>
            <Match when={step() === "preview"}>
              <PreviewStep
                source={selectedSource() as ImportSource}
                result={currentSourceResult}
                conflicts={conflicts}
                checkedFields={checkedFields}
                onToggleField={toggleField}
                onConflictChoice={setConflictChoice}
                onBack={backToSelect}
                onContinue={gotoConfirm}
                fieldConflictOf={isFieldConflicting}
              />
            </Match>
            <Match when={step() === "confirm"}>
              <ConfirmStep
                source={selectedSource() as ImportSource}
                fields={selectedFields()}
                conflicts={conflicts}
                fontFallback={fontFallback}
                applying={applying}
                onBack={() => setStep("preview")}
                onApply={applyImport}
              />
            </Match>
            <Match when={step() === "result"}>
              <ResultStep
                result={applyResult}
                fontFallback={fontFallback}
                onClose={props.onClose}
              />
            </Match>
          </Switch>
        </div>
      </div>
    </div>
  );
};

// ─── Step 1 · Source picker ─────────────────────────────────────────────

interface SelectStepProps {
  scanning: () => boolean;
  scanError: () => string | null;
  previewError: () => string | null;
  scanResults: () => ImportScanResult[];
  onChoose: (source: ImportSource) => void;
  onSkip: () => void;
  onRescan: () => void;
}

const SelectStep: Component<SelectStepProps> = (props) => {
  return (
    <div class="vs-config-import-select">
      <p class="vs-config-import-lede">
        Vibestation can import font, theme, shell, and keybindings from your
        existing terminal. Conflicts with built-in shortcuts (⌘T / ⌘W / ⌘D / ⌘⇧D
        / ⌘,) are flagged and never imported by default.
      </p>

      <Show when={props.scanError()}>
        <p class="vs-config-import-error" role="alert">
          Scan failed: {props.scanError()}
          <button
            type="button"
            class="vs-btn-secondary vs-config-import-rescan"
            onClick={props.onRescan}
          >
            Retry
          </button>
        </p>
      </Show>

      <Show when={props.previewError()}>
        <p class="vs-config-import-error" role="alert">
          Preview failed: {props.previewError()} · Try selecting a different
          source.
        </p>
      </Show>

      <Show when={props.scanning()}>
        <p class="vs-config-import-loading">Scanning default paths…</p>
      </Show>

      <Show when={!props.scanning() && !props.scanError()}>
        <ul class="vs-config-import-source-list">
          <For each={props.scanResults()}>
            {(result) => (
              <li
                classList={{
                  "vs-config-import-source": true,
                  detected: result.pathExists && result.errors.length === 0,
                  missing: !result.pathExists,
                  errored: result.pathExists && result.errors.length > 0,
                }}
              >
                <div class="vs-config-import-source-main">
                  <div class="vs-config-import-source-head">
                    <span class="vs-config-import-source-name">
                      {SOURCE_LABELS[result.source]}
                    </span>
                    <span class="vs-config-import-source-platform">
                      {SOURCE_PLATFORMS[result.source]}
                    </span>
                  </div>
                  <div class="vs-config-import-source-status">
                    <Show
                      when={result.pathExists}
                      fallback={
                        <span class="vs-config-import-source-missing">
                          Not detected
                        </span>
                      }
                    >
                      <Show when={result.errors.length === 0}>
                        <span class="vs-config-import-source-detected">
                          ✓ Detected · {result.detectedFields.length} fields
                        </span>
                      </Show>
                      <Show when={result.errors.length > 0}>
                        <span class="vs-config-import-source-errored">
                          ⚠ Parse error
                        </span>
                      </Show>
                    </Show>
                  </div>
                  <Show when={result.pathDisplay}>
                    <code class="vs-config-import-source-path">
                      {prettyPath(result.pathDisplay ?? "")}
                    </code>
                  </Show>
                  <Show when={result.errors.length > 0}>
                    <ul class="vs-config-import-error-list">
                      <For each={result.errors}>{(err) => <li>{err}</li>}</For>
                    </ul>
                  </Show>
                </div>
                <button
                  type="button"
                  class="vs-btn-primary vs-config-import-source-action"
                  disabled={!result.pathExists}
                  onClick={() => props.onChoose(result.source)}
                >
                  Use
                </button>
              </li>
            )}
          </For>
        </ul>
      </Show>

      <div class="vs-config-import-actions">
        <button type="button" class="vs-btn-secondary" onClick={props.onSkip}>
          Skip · configure manually
        </button>
      </div>
    </div>
  );
};

// ─── Step 2 · Preview list ──────────────────────────────────────────────

interface PreviewStepProps {
  source: ImportSource;
  result: () => ImportScanResult | null;
  conflicts: () => KeyBindingConflict[];
  checkedFields: () => Set<number>;
  onToggleField: (idx: number) => void;
  onConflictChoice: (sourceKey: string, choice: KeyBindingResolution) => void;
  onBack: () => void;
  onContinue: () => void;
  fieldConflictOf: (field: ImportFieldType) => KeyBindingConflict | null;
}

const PreviewStep: Component<PreviewStepProps> = (props) => {
  const fieldOrder = (kind: ImportFieldType["kind"]): number => {
    // spec §A.3 字段顺序：font_family → font_size → theme → shell → keybindings
    switch (kind) {
      case "fontFamily":
        return 0;
      case "fontSize":
        return 1;
      case "theme":
        return 2;
      case "shell":
        return 3;
      case "ansiColor":
        return 4;
      case "keyBinding":
        return 5;
      default:
        return 99;
    }
  };

  const sortedFields = createMemo(() => {
    const r = props.result();
    if (!r) return [] as Array<{ field: ImportFieldType; idx: number }>;
    return r.detectedFields
      .map((field, idx) => ({ field, idx }))
      .sort((a, b) => fieldOrder(a.field.kind) - fieldOrder(b.field.kind));
  });

  return (
    <div class="vs-config-import-preview">
      <p class="vs-config-import-lede">
        Importing from <strong>{SOURCE_LABELS[props.source]}</strong>. Review
        the fields below — only checked items will be applied. Conflicts with
        Vibestation built-ins are highlighted in yellow.
      </p>

      <Show
        when={(props.result()?.detectedFields.length ?? 0) > 0}
        fallback={
          <p class="vs-config-import-empty">
            No importable fields detected. Try a different source.
          </p>
        }
      >
        <ul class="vs-config-import-field-list">
          <For each={sortedFields()}>
            {(entry) => (
              <FieldRow
                field={entry.field}
                idx={entry.idx}
                checked={props.checkedFields().has(entry.idx)}
                conflict={props.fieldConflictOf(entry.field)}
                onToggle={() => props.onToggleField(entry.idx)}
                onConflictChoice={props.onConflictChoice}
              />
            )}
          </For>
        </ul>
      </Show>

      <div class="vs-config-import-actions">
        <button type="button" class="vs-btn-secondary" onClick={props.onBack}>
          Back
        </button>
        <button
          type="button"
          class="vs-btn-primary"
          onClick={props.onContinue}
          disabled={props.checkedFields().size === 0}
        >
          Continue
        </button>
      </div>
    </div>
  );
};

interface FieldRowProps {
  field: ImportFieldType;
  idx: number;
  checked: boolean;
  conflict: KeyBindingConflict | null;
  onToggle: () => void;
  onConflictChoice: (sourceKey: string, choice: KeyBindingResolution) => void;
}

const FieldRow: Component<FieldRowProps> = (props) => {
  // Issue #205 Finding 2 (a) · v0.1 disabled 字段（AnsiColor）的可视提示 + tooltip
  const disabled = () => isFieldDisabled(props.field);
  const disabledReason = () => getDisabledReason(props.field);

  return (
    <li
      classList={{
        "vs-config-import-field": true,
        "vs-config-import-field-conflict": props.conflict !== null,
        "vs-config-import-field-disabled": disabled(),
      }}
    >
      <label class="vs-config-import-field-label" title={disabledReason()}>
        <input
          type="checkbox"
          checked={props.checked}
          onChange={props.onToggle}
          disabled={disabled()}
        />
        <span class="vs-config-import-field-meta">
          <span class="vs-config-import-field-kind">
            {fieldKindLabel(props.field.kind)}
          </span>
          <span class="vs-config-import-field-value">
            {fieldValueDisplay(props.field)}
          </span>
          <Show when={disabled()}>
            <span
              class="vs-config-import-field-badge"
              aria-label={disabledReason()}
            >
              v0.2
            </span>
          </Show>
        </span>
      </label>

      <Show when={props.conflict}>
        {(conflict) => (
          <div class="vs-config-import-conflict-banner" role="alert">
            <span class="vs-config-import-conflict-icon" aria-hidden="true">
              ⚠
            </span>
            <span class="vs-config-import-conflict-msg">
              <strong>{conflict().vibeKey}</strong> already maps to{" "}
              <code>{conflict().vibeAction}</code> in Vibestation.
            </span>
            <div class="vs-config-import-conflict-choice">
              <label>
                <input
                  type="radio"
                  name={`conflict-${conflict().sourceKey}`}
                  checked={conflict().userChoice === "keepVibe"}
                  onChange={() =>
                    props.onConflictChoice(conflict().sourceKey, "keepVibe")
                  }
                />
                Keep Vibestation
              </label>
              <label>
                <input
                  type="radio"
                  name={`conflict-${conflict().sourceKey}`}
                  checked={conflict().userChoice === "override"}
                  onChange={() =>
                    props.onConflictChoice(conflict().sourceKey, "override")
                  }
                />
                Override (v0.1 stored only · activates in v0.2+)
              </label>
            </div>
          </div>
        )}
      </Show>
    </li>
  );
};

// ─── Step 3 · Confirm ───────────────────────────────────────────────────

interface ConfirmStepProps {
  source: ImportSource;
  fields: ImportFieldType[];
  conflicts: () => KeyBindingConflict[];
  fontFallback: () => FontFallback[];
  applying: () => boolean;
  onBack: () => void;
  onApply: () => void;
}

const ConfirmStep: Component<ConfirmStepProps> = (props) => {
  const overrideCount = () =>
    props.conflicts().filter((c) => c.userChoice === "override").length;
  const keepCount = () =>
    props.conflicts().filter((c) => c.userChoice === "keepVibe").length;
  const missingFonts = () => props.fontFallback().filter((f) => !f.available);

  return (
    <div class="vs-config-import-confirm">
      <p class="vs-config-import-lede">
        Ready to apply <strong>{props.fields.length}</strong> fields from{" "}
        <strong>{SOURCE_LABELS[props.source]}</strong>.
      </p>

      <ul class="vs-config-import-summary">
        <Show when={props.fields.length > 0}>
          <li>
            <span class="vs-config-import-summary-label">Apply</span>
            <span>{summarizeFields(props.fields)}</span>
          </li>
        </Show>
        <Show when={keepCount() > 0}>
          <li class="vs-config-import-summary-warn">
            <span class="vs-config-import-summary-label">Skip</span>
            <span>
              {keepCount()} conflicting keybinding
              {keepCount() === 1 ? "" : "s"} (Vibestation built-in kept)
            </span>
          </li>
        </Show>
        <Show when={overrideCount() > 0}>
          <li>
            <span class="vs-config-import-summary-label">Override</span>
            <span>
              {overrideCount()} keybinding
              {overrideCount() === 1 ? "" : "s"} stored for v0.2+ activation
            </span>
          </li>
        </Show>
        <Show when={missingFonts().length > 0}>
          <li class="vs-config-import-summary-warn">
            <span class="vs-config-import-summary-label">Font fallback</span>
            <span>
              <For each={missingFonts()}>
                {(f) => (
                  <code class="vs-config-import-missing-font">{f.font}</code>
                )}
              </For>
              not installed · will fall back to JetBrains Mono.
            </span>
          </li>
        </Show>
      </ul>

      <div class="vs-config-import-actions">
        <button
          type="button"
          class="vs-btn-secondary"
          onClick={props.onBack}
          disabled={props.applying()}
        >
          Back
        </button>
        <button
          type="button"
          class="vs-btn-primary"
          onClick={props.onApply}
          disabled={props.applying() || props.fields.length === 0}
        >
          {props.applying() ? "Applying…" : "Apply"}
        </button>
      </div>
    </div>
  );
};

// ─── Step 4 · Result ────────────────────────────────────────────────────

interface ResultStepProps {
  result: () => ImportApplyResult | null;
  fontFallback: () => FontFallback[];
  onClose: () => void;
}

const ResultStep: Component<ResultStepProps> = (props) => {
  const r = () => props.result();

  return (
    <div class="vs-config-import-result">
      <Show
        when={r() && r()!.errors.length === 0}
        fallback={<ResultErrors result={r} />}
      >
        <p class="vs-config-import-success">✓ Import applied.</p>
      </Show>

      <Show when={r() && r()!.applied.length > 0}>
        <section>
          <h3>Updated settings</h3>
          <ul class="vs-config-import-applied">
            <For each={r()!.applied}>
              {(field) => (
                <li>
                  <code>{prettyApplied(field)}</code>
                </li>
              )}
            </For>
          </ul>
        </section>
      </Show>

      <Show when={r() && r()!.skippedConflicts.length > 0}>
        <section>
          <h3>Skipped (Vibestation built-ins kept)</h3>
          <ul class="vs-config-import-skipped">
            <For each={r()!.skippedConflicts}>
              {(c) => (
                <li>
                  <code>{c.vibeKey}</code> → <code>{c.vibeAction}</code>
                </li>
              )}
            </For>
          </ul>
        </section>
      </Show>

      <Show when={props.fontFallback().filter((f) => !f.available).length > 0}>
        <p class="vs-config-import-font-note">
          Some fonts were not installed on this machine and fell back to
          JetBrains Mono. Install the original font and re-apply if you want the
          imported family.
        </p>
      </Show>

      <div class="vs-config-import-actions">
        <button type="button" class="vs-btn-primary" onClick={props.onClose}>
          Done
        </button>
      </div>
    </div>
  );
};

const ResultErrors: Component<{
  result: () => ImportApplyResult | null;
}> = (props) => (
  <Show when={props.result()}>
    {(r) => (
      <Show
        when={r().errors.length > 0}
        fallback={
          <p class="vs-config-import-success">
            ✓ Nothing applied (no fields selected).
          </p>
        }
      >
        <p class="vs-config-import-error" role="alert">
          Apply finished with errors:
        </p>
        <ul class="vs-config-import-error-list">
          <For each={r().errors}>{(e) => <li>{e}</li>}</For>
        </ul>
      </Show>
    )}
  </Show>
);

// ─── helpers ────────────────────────────────────────────────────────────

function safeFontCheck(family: string): boolean {
  // 容错：浏览器拒绝某些 family 时 throw · 把无法判定当作"不可用"以触发 fallback
  try {
    return document.fonts.check(`12px "${escapeCss(family)}"`);
  } catch {
    return false;
  }
}

function escapeCss(value: string): string {
  return value.replace(/["\\]/g, "\\$&");
}

function fieldKindLabel(kind: ImportFieldType["kind"]): string {
  switch (kind) {
    case "fontFamily":
      return "Font family";
    case "fontSize":
      return "Font size";
    case "theme":
      return "Theme";
    case "shell":
      return "Shell";
    case "keyBinding":
      return "Keybinding";
    case "ansiColor":
      return "ANSI color";
  }
}

function fieldValueDisplay(field: ImportFieldType): string {
  switch (field.kind) {
    case "fontFamily":
      return field.value;
    case "fontSize":
      return `${field.value}pt`;
    case "theme":
      return field.value;
    case "shell":
      return field.value;
    case "keyBinding":
      return `${field.key} → ${field.action}`;
    case "ansiColor":
      return `Ansi ${field.index} · ${field.hex}`;
  }
}

function summarizeFields(fields: ImportFieldType[]): string {
  const counts = new Map<string, number>();
  for (const f of fields) {
    const k = fieldKindLabel(f.kind);
    counts.set(k, (counts.get(k) ?? 0) + 1);
  }
  return Array.from(counts.entries())
    .map(([label, count]) => (count === 1 ? label : `${count} ${label}`))
    .join(" · ");
}

function prettyApplied(field: AppliedField): string {
  // Issue #206 Advisory #7 修复：AppliedField 是 ts-rs 生成的 union literal type
  // · 编译期 exhaustive · 拼错 case label 直接 TS error
  switch (field) {
    case "fontFamily":
      return "Font family";
    case "fontSize":
      return "Font size";
    case "theme":
      return "Theme";
    case "defaultShell":
      return "Default shell";
    case "importedKeybindings":
      return "Imported keybindings (stored for v0.2+)";
  }
}

function prettyPath(p: string): string {
  // Issue #206 Advisory #3 修复：后端 ImportScanResult.path_display 已经用 $HOME
  // 准确替换家目录为 `~/...`（见 ipc.rs prettify_home_path）· 前端不再需要 hard-coded
  // 路径模式 regex。保留此函数作 fallback safety + 防 backend 旧实现返回原始 path。
  return p.replace(/^(\/Users\/[^/]+|\/home\/[^/]+|\/root)(?=\/|$)/, "~");
}
