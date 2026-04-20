export interface LayoutState {
  primaryOpen: boolean;
  secondaryOpen: boolean;
  bottomOpen: boolean;
  primaryWidth: number;
  secondaryWidth: number;
  bottomHeight: number;
}

export const DEFAULT_LAYOUT: LayoutState = {
  primaryOpen: true,
  secondaryOpen: false,
  bottomOpen: false,
  primaryWidth: 236,
  secondaryWidth: 400,
  bottomHeight: 240,
};

export type LayoutAction =
  | { kind: "toggle-primary" }
  | { kind: "toggle-secondary" }
  | { kind: "toggle-bottom" }
  | { kind: "resize-primary"; width: number }
  | { kind: "resize-secondary"; width: number }
  | { kind: "resize-bottom"; height: number }
  | { kind: "reset-primary" }
  | { kind: "reset-secondary" }
  | { kind: "reset-bottom" }
  | { kind: "restore"; state: LayoutState };

const MIN_PRIMARY = 200;
const MAX_PRIMARY = 400;
const MIN_SECONDARY = 240;
const MAX_SECONDARY = 500;
const MIN_BOTTOM = 150;
const MAX_BOTTOM = 500;

export function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value));
}

export function layoutReducer(
  state: LayoutState,
  action: LayoutAction,
): LayoutState {
  switch (action.kind) {
    case "toggle-primary":
      return { ...state, primaryOpen: !state.primaryOpen };
    case "toggle-secondary":
      return { ...state, secondaryOpen: !state.secondaryOpen };
    case "toggle-bottom":
      return { ...state, bottomOpen: !state.bottomOpen };
    case "resize-primary":
      return {
        ...state,
        primaryWidth: clamp(action.width, MIN_PRIMARY, MAX_PRIMARY),
      };
    case "resize-secondary":
      return {
        ...state,
        secondaryWidth: clamp(action.width, MIN_SECONDARY, MAX_SECONDARY),
      };
    case "resize-bottom":
      return {
        ...state,
        bottomHeight: clamp(action.height, MIN_BOTTOM, MAX_BOTTOM),
      };
    case "reset-primary":
      return { ...state, primaryWidth: DEFAULT_LAYOUT.primaryWidth };
    case "reset-secondary":
      return { ...state, secondaryWidth: DEFAULT_LAYOUT.secondaryWidth };
    case "reset-bottom":
      return { ...state, bottomHeight: DEFAULT_LAYOUT.bottomHeight };
    case "restore":
      return action.state;
  }
}
