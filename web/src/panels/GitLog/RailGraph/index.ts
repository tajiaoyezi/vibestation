// MVP-12 Phase A · RailGraph data layer public API
export type { RailGraphInputCommit, RailLaneAssignment } from "./types";
export { ENABLE_RAIL_GRAPH } from "./types";
export { buildRailGraphInputFromGitLog } from "./build-input";
export { allocateLanes } from "./lane-allocator";
export { normalizeRefs } from "./refs-normalize";
export { branchNameToColorKey } from "./color-mapper";
export { RailGraphCanvas } from "./RailGraphCanvas";
export type { RailGraphCanvasProps } from "./RailGraphCanvas";
