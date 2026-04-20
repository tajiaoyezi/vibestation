import { type Component } from "solid-js";

interface SecondarySidebarProps {
  layout: () => { secondaryOpen: boolean; secondaryWidth: number };
  onResizeStart: (e: MouseEvent) => void;
  onResizeReset: () => void;
}

export const SecondarySidebar: Component<SecondarySidebarProps> = (props) => {
  return (
    <div
      class={`vs-secondary-sidebar${props.layout().secondaryOpen ? "" : " vs-panel-hidden"}`}
      role="complementary"
      aria-label="Secondary sidebar"
      style={{
        width: props.layout().secondaryOpen
          ? `${props.layout().secondaryWidth}px`
          : "0",
      }}
    >
      <div class="vs-panel-head">
        <span class="vs-panel-title">Git Log</span>
        <div class="vs-panel-actions">
          <span class="vs-kbd-tip">⌘2</span>
        </div>
      </div>
      <div class="vs-panel-body">
        <p class="vs-placeholder-text">MVP-07 will fill this area</p>
      </div>
      <div
        class="vs-resize-handle vs-resize-handle-w"
        role="separator"
        aria-orientation="vertical"
        aria-label="Resize secondary sidebar"
        onMouseDown={props.onResizeStart}
        onDblClick={props.onResizeReset}
      />
    </div>
  );
};
