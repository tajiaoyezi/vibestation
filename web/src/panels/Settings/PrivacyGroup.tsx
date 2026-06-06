import { createSignal, createResource, Show, type Component } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { useSettings } from "../../stores/settings";
import type { TelemetryStatus } from "../../bindings";
import { t, normalizeLanguage } from "../../i18n";

export const PrivacyGroup: Component = () => {
  const { settings, updateSettings } = useSettings();
  const language = () => normalizeLanguage(settings.language);
  const label = (key: string) => t(key, language());
  const [showDetails, setShowDetails] = createSignal(false);
  const [copied, setCopied] = createSignal(false);

  // §C.4 · 收集端点 host 显示（DSN 的 host 部分 · 不含 public_key / project_id）
  const [status, { refetch: refetchStatus }] = createResource<TelemetryStatus>(
    () => invoke<TelemetryStatus>("telemetry_status_get"),
  );

  const telemetryLabel = () => {
    if (settings.telemetryOptIn === null) {
      return label("settings.privacy.notDecided");
    }
    return settings.telemetryOptIn
      ? label("settings.privacy.enabled")
      : label("settings.privacy.disabled");
  };

  const copyEndpoint = async () => {
    const host = status()?.endpointHost;
    if (!host) return;
    await navigator.clipboard.writeText(host);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1500);
  };

  return (
    <div class="vs-settings-fields">
      <div class="vs-settings-field vs-settings-field--row">
        <div class="vs-settings-field-labels">
          <span class="vs-settings-label">
            {label("settings.privacy.telemetry")}
          </span>
          <span class="vs-settings-sublabel">{telemetryLabel()}</span>
        </div>
        <button
          type="button"
          class="vs-settings-toggle"
          classList={{
            active: settings.telemetryOptIn === true,
            neutral: settings.telemetryOptIn === null,
          }}
          onClick={async () => {
            const next =
              settings.telemetryOptIn === null
                ? true
                : settings.telemetryOptIn
                  ? false
                  : true;
            await updateSettings({ telemetryOptIn: next });
            // SDK init 状态可能随 opt-in 切换 · 同步刷新
            await refetchStatus();
          }}
          aria-pressed={settings.telemetryOptIn ?? "mixed"}
          role="switch"
          title={label("settings.privacy.toggleTelemetry")}
        >
          <span class="vs-settings-toggle-knob" />
        </button>
      </div>

      {/* §C.4 · 收集端点公开显示 + 一键复制 */}
      <div class="vs-settings-field vs-settings-field--row">
        <div class="vs-settings-field-labels">
          <span class="vs-settings-label">
            {label("settings.privacy.collectionEndpoint")}
          </span>
          <span class="vs-settings-sublabel vs-settings-endpoint-host">
            {status()?.endpointHost ?? label("settings.privacy.loading")}
          </span>
        </div>
        <button
          type="button"
          class="vs-settings-copy-btn"
          onClick={copyEndpoint}
          disabled={!status() || status()?.endpointHost === "Not configured"}
          aria-label={label("settings.privacy.copyCollectionEndpoint")}
        >
          {copied()
            ? label("settings.privacy.copied")
            : label("settings.privacy.copy")}
        </button>
      </div>

      <button
        type="button"
        class="vs-settings-link"
        onClick={() => setShowDetails(true)}
      >
        {label("settings.privacy.viewWhatWeCollect")}
      </button>

      <Show when={showDetails()}>
        <div
          class="vs-settings-detail-overlay"
          role="dialog"
          aria-modal="true"
          onClick={() => setShowDetails(false)}
        >
          <div
            class="vs-settings-detail-card"
            onClick={(e) => e.stopPropagation()}
          >
            <div class="vs-settings-detail-header">
              <h3>{label("settings.privacy.dataCollectionSummary")}</h3>
              <button
                type="button"
                class="vs-settings-close"
                onClick={() => setShowDetails(false)}
                aria-label={label("settings.privacy.closeDetails")}
              >
                ✕
              </button>
            </div>

            <div class="vs-settings-detail-body">
              <section>
                <h4>We collect</h4>
                <ul>
                  <li>Anonymous crash reports (stack trace hash)</li>
                  <li>App version number</li>
                  <li>OS type (macOS / Linux)</li>
                </ul>
              </section>

              <section>
                <h4>We do NOT collect</h4>
                <ul>
                  <li>IP address</li>
                  <li>Personal file paths</li>
                  <li>Commit messages or repository names</li>
                  <li>Terminal content or commands</li>
                  <li>Any personally identifiable information</li>
                </ul>
              </section>

              <p class="vs-settings-detail-note">
                You can change this anytime in Preferences → Privacy. Full
                details in{" "}
                <a
                  href="https://github.com/tajiaoyezi/vibestation/blob/main/privacy-policy.md"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="vs-settings-link-inline"
                >
                  privacy-policy.md
                </a>
                .
              </p>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};
