/**
 * PaneSplitter · MVP-05 Phase C
 *
 * 1px 浅色分隔条 · hover 主色 + resize 光标。
 * §D 拖拽改 ratio + 双击复位 50/50 留给 PR #145 实施（drag handler + rAF）。
 * 当前 PR #144 仅展示静态 splitter（visual placeholder）。
 */
import { type Component } from "solid-js";
import type { SplitDir } from "../../bindings";

type PaneSplitterProps = {
  direction: SplitDir;
  ratio: number;
  parentPaneId: string;
  onDragEnd?: (parentPaneId: string, ratio: number) => void;
};

export const PaneSplitter: Component<PaneSplitterProps> = (props) => {
  const handleDoubleClick = () => {
    // §D 双击复位 50/50
    props.onDragEnd?.(props.parentPaneId, 0.5);
  };

  return (
    <div
      class={`vs-pane-splitter vs-pane-splitter-${props.direction}`}
      onDblClick={handleDoubleClick}
      role="separator"
      aria-orientation={
        props.direction === "horizontal" ? "vertical" : "horizontal"
      }
      aria-valuenow={Math.round(props.ratio * 100)}
      tabindex={-1}
    />
  );
};
