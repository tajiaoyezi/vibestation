import { describe, expect, it } from "vitest";
import { t } from "../../src/i18n";

describe("FEAT-02 first Settings chrome copy", () => {
  it("TEST-FEAT-02.6: settings shell labels translate", () => {
    expect(t("settings.title", "en")).toBe("Preferences");
    expect(t("settings.title", "zh-Hans")).toBe("偏好设置");
    expect(t("settings.import", "en")).toBe("Import...");
    expect(t("settings.import", "zh-Hans")).toBe("导入...");
    expect(t("settings.close", "en")).toBe("Close settings");
    expect(t("settings.close", "zh-Hans")).toBe("关闭设置");
  });

  it("TEST-FEAT-02.6: settings group labels translate", () => {
    expect(t("settings.groups.appearance", "en")).toBe("Appearance");
    expect(t("settings.groups.appearance", "zh-Hans")).toBe("外观");
    expect(t("settings.groups.terminal", "en")).toBe("Terminal");
    expect(t("settings.groups.terminal", "zh-Hans")).toBe("终端");
    expect(t("settings.groups.externalTerminal", "en")).toBe(
      "External Terminal",
    );
    expect(t("settings.groups.externalTerminal", "zh-Hans")).toBe("外部终端");
  });

  it("TEST-FEAT-02.6: appearance labels translate", () => {
    expect(t("settings.appearance.theme", "zh-Hans")).toBe("主题");
    expect(t("settings.appearance.auto", "zh-Hans")).toBe("自动");
    expect(t("settings.appearance.fontFamily", "zh-Hans")).toBe("字体");
    expect(t("settings.appearance.backgroundOpacity", "zh-Hans")).toBe(
      "背景不透明度",
    );
    expect(t("settings.appearance.windowPaddingX", "zh-Hans")).toBe(
      "窗口水平内边距",
    );
    expect(t("settings.appearance.cursorBlink", "zh-Hans")).toBe("光标闪烁");
  });

  it("TEST-FEAT-02.6: remaining settings control labels translate", () => {
    expect(t("settings.terminal.defaultShell", "en")).toBe("Default shell");
    expect(t("settings.terminal.defaultShell", "zh-Hans")).toBe("默认 Shell");
    expect(t("settings.terminal.pasteProtection", "zh-Hans")).toBe("粘贴保护");
    expect(t("settings.terminal.ptyWarmPool", "zh-Hans")).toBe("PTY 预热池");
    expect(t("settings.terminal.poolSize", "zh-Hans")).toBe("池大小");

    expect(t("settings.externalTerminal.preferredTerminal", "en")).toBe(
      "Preferred terminal",
    );
    expect(t("settings.externalTerminal.preferredTerminal", "zh-Hans")).toBe(
      "首选终端",
    );
    expect(t("settings.externalTerminal.askEveryTime", "zh-Hans")).toBe(
      "每次询问",
    );
    expect(t("settings.externalTerminal.dontAskAgain", "zh-Hans")).toBe(
      "不要再次询问",
    );

    expect(t("settings.git.userName", "zh-Hans")).toBe("用户名");
    expect(t("settings.git.userEmail", "zh-Hans")).toBe("用户邮箱");
    expect(t("settings.git.fromGitConfig", "zh-Hans")).toBe("来自 git config");

    expect(t("settings.privacy.telemetry", "zh-Hans")).toBe("遥测");
    expect(t("settings.privacy.notDecided", "zh-Hans")).toBe("未决定");
    expect(t("settings.privacy.collectionEndpoint", "zh-Hans")).toBe(
      "收集端点",
    );
    expect(t("settings.privacy.viewWhatWeCollect", "zh-Hans")).toBe(
      "查看收集内容",
    );
  });

  it("TEST-FEAT-02.6: workspace chrome labels translate", () => {
    expect(t("chrome.sidebars.primary", "en")).toBe("Primary sidebar");
    expect(t("chrome.sidebars.primary", "zh-Hans")).toBe("主侧边栏");
    expect(t("chrome.sidebars.workspaces", "en")).toBe("Workspaces");
    expect(t("chrome.sidebars.workspaces", "zh-Hans")).toBe("工作区");
    expect(t("chrome.sidebars.noWorkspacesYet", "zh-Hans")).toBe(
      "还没有工作区。",
    );
    expect(t("chrome.sidebars.importSettings", "zh-Hans")).toBe(
      "从其他终端导入设置",
    );
    expect(t("chrome.sidebars.createWorkspace", "zh-Hans")).toBe("创建工作区");

    expect(t("chrome.activity.gitLog", "zh-Hans")).toBe("Git 日志");
    expect(t("chrome.activity.gitStatus", "zh-Hans")).toBe("Git 状态");
    expect(t("chrome.activity.panelToggles", "zh-Hans")).toBe("面板切换");

    expect(t("chrome.bottom.panel", "zh-Hans")).toBe("底部面板");
    expect(t("chrome.bottom.resizePanel", "zh-Hans")).toBe("调整底部面板大小");
    expect(t("chrome.bottom.output", "zh-Hans")).toBe("输出");
    expect(t("chrome.bottom.diff", "zh-Hans")).toBe("差异");

    expect(t("chrome.topbar.togglePrimarySidebar", "zh-Hans")).toBe(
      "切换主侧边栏",
    );
    expect(t("chrome.window.minimize", "zh-Hans")).toBe("最小化");
    expect(t("chrome.window.maximize", "zh-Hans")).toBe("最大化");
    expect(t("chrome.window.restore", "zh-Hans")).toBe("还原");
    expect(t("chrome.window.close", "zh-Hans")).toBe("关闭");

    expect(t("chrome.status.statusBar", "zh-Hans")).toBe("状态栏");
    expect(t("chrome.status.remote", "zh-Hans")).toBe("远端");
    expect(t("chrome.status.merge", "zh-Hans")).toBe("合并");
    expect(t("chrome.status.openSettings", "zh-Hans")).toBe("打开设置");
    expect(t("chrome.status.dismissError", "zh-Hans")).toBe("关闭错误");
    expect(t("chrome.status.ipcConnecting", "zh-Hans")).toBe("ipc：连接中...");
    expect(t("chrome.status.ipcOk", "zh-Hans")).toBe("ipc：");
    expect(t("chrome.status.ipcError", "zh-Hans")).toBe("ipc 错误：");
    expect(t("chrome.status.alpha", "zh-Hans")).toBe("alpha");
  });

  it("TEST-FEAT-02.6: remaining main and dialog chrome labels translate", () => {
    expect(t("chrome.main.contentArea", "en")).toBe("Main content area");
    expect(t("chrome.main.contentArea", "zh-Hans")).toBe("主内容区");
    expect(t("chrome.main.emptyWorkspace", "zh-Hans")).toBe(
      "选择或创建工作区以开始",
    );
    expect(t("chrome.main.diff", "zh-Hans")).toBe("差异");
    expect(t("chrome.main.backToTerminal", "zh-Hans")).toBe("返回终端");
    expect(t("chrome.sidebars.secondary", "zh-Hans")).toBe("副侧边栏");
    expect(t("chrome.sidebars.resizeSecondary", "zh-Hans")).toBe(
      "调整副侧边栏大小",
    );

    expect(t("dialogs.telemetry.title", "zh-Hans")).toBe(
      "帮助改进 Vibestation",
    );
    expect(t("dialogs.telemetry.decline", "zh-Hans")).toBe("拒绝");
    expect(t("dialogs.telemetry.accept", "zh-Hans")).toBe("接受");

    expect(t("dialogs.popToExternal.title", "zh-Hans")).toBe("在外部终端打开");
    expect(t("dialogs.popToExternal.close", "zh-Hans")).toBe("关闭对话框");
    expect(t("dialogs.popToExternal.retry", "zh-Hans")).toBe("重试");
    expect(t("dialogs.popToExternal.detectingTerminals", "zh-Hans")).toBe(
      "正在检测终端...",
    );
    expect(t("dialogs.popToExternal.noTerminals", "zh-Hans")).toBe(
      "未检测到外部终端。",
    );
    expect(t("dialogs.popToExternal.dontAskAgain", "zh-Hans")).toBe(
      "不要再次询问",
    );
    expect(t("dialogs.popToExternal.cancel", "zh-Hans")).toBe("取消");

    expect(t("dialogs.branchSwitcher.title", "zh-Hans")).toBe("切换分支");
    expect(t("dialogs.branchSwitcher.loadingBranches", "zh-Hans")).toBe(
      "正在加载分支...",
    );
    expect(t("dialogs.branchSwitcher.noBranchMatched", "zh-Hans")).toBe(
      "没有匹配的分支",
    );
    expect(t("dialogs.branchSwitcher.notGitWorkspace", "zh-Hans")).toBe(
      "当前工作区不是 Git 仓库",
    );
    expect(t("dialogs.branchSwitcher.retry", "zh-Hans")).toBe("重试");
  });

  it("TEST-FEAT-02.6: remaining git dialog chrome labels translate", () => {
    expect(t("dialogs.common.cancel", "zh-Hans")).toBe("取消");
    expect(t("dialogs.common.confirm", "zh-Hans")).toBe("确认");
    expect(t("dialogs.common.continue", "zh-Hans")).toBe("继续");

    expect(t("dialogs.createBranch.title", "en")).toBe("Create branch");
    expect(t("dialogs.createBranch.title", "zh-Hans")).toBe("创建分支");
    expect(t("dialogs.createBranch.name", "zh-Hans")).toBe("名称");
    expect(t("dialogs.createBranch.from", "zh-Hans")).toBe("来源");
    expect(t("dialogs.createBranch.createAndCheckout", "zh-Hans")).toBe(
      "创建并切换",
    );
    expect(t("dialogs.createBranch.creating", "zh-Hans")).toBe("创建中...");

    expect(t("dialogs.cherryPick.close", "zh-Hans")).toBe(
      "关闭 cherry-pick 对话框",
    );
    expect(t("dialogs.cherryPick.autoCommitEach", "zh-Hans")).toBe(
      "逐个自动提交",
    );
    expect(t("dialogs.cherryPick.action", "zh-Hans")).toBe("Cherry-pick");
    expect(t("dialogs.cherryPick.submitting", "zh-Hans")).toBe(
      "Cherry-pick 中...",
    );

    expect(t("dialogs.merge.close", "zh-Hans")).toBe("关闭合并对话框");
    expect(t("dialogs.merge.sourceBranch", "zh-Hans")).toBe("源分支");
    expect(t("dialogs.merge.loadingBranches", "zh-Hans")).toBe(
      "正在加载分支...",
    );
    expect(t("dialogs.merge.searchBranch", "zh-Hans")).toBe("搜索分支");
    expect(t("dialogs.merge.commitMessage", "zh-Hans")).toBe("提交信息");
    expect(t("dialogs.merge.action", "zh-Hans")).toBe("合并");
    expect(t("dialogs.merge.submitting", "zh-Hans")).toBe("合并中...");

    expect(t("dialogs.remoteSelector.pushTitle", "en")).toBe(
      "Select push remote",
    );
    expect(t("dialogs.remoteSelector.pullTitle", "zh-Hans")).toBe(
      "选择 pull 远端",
    );
    expect(t("dialogs.remoteSelector.fetchTitle", "zh-Hans")).toBe(
      "Fetch 远端",
    );
    expect(t("dialogs.remoteSelector.remoteRefs", "zh-Hans")).toBe("远端 refs");
    expect(t("dialogs.remoteSelector.pruneDeletedRefs", "zh-Hans")).toBe(
      "清理已删除的 refs",
    );
  });
});
