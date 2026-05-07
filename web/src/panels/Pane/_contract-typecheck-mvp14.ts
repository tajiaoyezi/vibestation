// MVP-14 Phase A · H2 contract 哨兵（type-only · 不被任何代码 import）
// 作用：让 ts-rs binding 字段 rename / removal 立即触发 typecheck FAIL
// 仿 PR #256 droid `web/src/panels/GitLog/RailGraph/_contract-typecheck.ts` 模式

import type { LayoutEnvelope } from "../../bindings/LayoutEnvelope";
import type { LayoutPresetKind } from "../../bindings/LayoutPresetKind";
import type { LayoutApplyAdvancedRequest } from "../../bindings/LayoutApplyAdvancedRequest";
import type { LayoutApplyResult } from "../../bindings/LayoutApplyResult";
import type { PaneNavigateRequest } from "../../bindings/PaneNavigateRequest";
import type { PaneNavigateResult } from "../../bindings/PaneNavigateResult";
import type { PaneMaximizeRequest } from "../../bindings/PaneMaximizeRequest";
import type { PaneMaximizeResult } from "../../bindings/PaneMaximizeResult";
import type { PaneResizeStepRequest } from "../../bindings/PaneResizeStepRequest";
import type { LayoutHistoryEntry } from "../../bindings/LayoutHistoryEntry";
import type { WorkspaceLayoutState } from "../../bindings/WorkspaceLayoutState";
import type { PaneNavDirection } from "../../bindings/PaneNavDirection";

const _envelope: Pick<LayoutEnvelope, "version" | "root" | "focusedPaneId" | "updatedAt"> = {} as never;
const _preset: LayoutPresetKind = "solo" as LayoutPresetKind;
const _apply: Pick<LayoutApplyAdvancedRequest, "tabId" | "preset" | "preserveInstances" | "confirmed"> = {} as never;
const _applyResult: Pick<LayoutApplyResult, "response" | "reusedPaneIds" | "createdPaneIds" | "closedPaneIds"> = {} as never;
const _nav: Pick<PaneNavigateRequest, "tabId" | "fromPaneId" | "direction"> = {} as never;
const _navResult: PaneNavigateResult = {} as never;
const _max: Pick<PaneMaximizeRequest, "tabId" | "paneId" | "toggle"> = {} as never;
const _maxResult: PaneMaximizeResult = {} as never;
const _resize: PaneResizeStepRequest = {} as never;
const _hist: LayoutHistoryEntry = {} as never;
const _wsState: WorkspaceLayoutState = {} as never;
const _navDir: PaneNavDirection = "left" as PaneNavDirection;

export type _Mvp14ContractAnchor = typeof _envelope;
