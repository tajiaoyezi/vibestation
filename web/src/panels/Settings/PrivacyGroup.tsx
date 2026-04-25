import { createSignal, Show, type Component } from "solid-js";
import { useSettings } from "../../stores/settings";

export const PrivacyGroup: Component = () => {
  const { settings, updateSettings } = useSettings();
  const [showDetails, setShowDetails] = createSignal(false);

  const telemetryLabel = () => {
    if (settings.telemetryOptIn === null) return "Not decided";
    return settings.telemetryOptIn ? "Enabled" : "Disabled";
  };

  return (
    <div class="vs-settings-fields">
      <div class="vs-settings-field vs-settings-field--row">
        <div class="vs-settings-field-labels">
          <span class="vs-settings-label">Telemetry</span>
          <span class="vs-settings-sublabel">{telemetryLabel()}</span>
        </div>
        <button
          type="button"
          class="vs-settings-toggle"
          classList={{
            active: settings.telemetryOptIn === true,
            neutral: settings.telemetryOptIn === null,
          }}
          onClick={() => {
            const next =
              settings.telemetryOptIn === null
                ? true
                : settings.telemetryOptIn
                  ? false
                  : true;
            updateSettings({ telemetryOptIn: next });
          }}
          aria-pressed={settings.telemetryOptIn ?? "mixed"}
          role="switch"
          title="Toggle telemetry"
        >
          <span class="vs-settings-toggle-knob" />
        </button>
      </div>

      <button
        type="button"
        class="vs-settings-link"
        onClick={() => setShowDetails(true)}
      >
        View what we collect
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
              <h3>Data collection summary</h3>
              <button
                type="button"
                class="vs-settings-close"
                onClick={() => setShowDetails(false)}
                aria-label="Close"
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
                <a href="#" class="vs-settings-link-inline">
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
