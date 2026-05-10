// MVP-12 Phase A · RailGraph data layer public API
export type { RailGraphInputCommit, RailLaneAssignment } from "./types";
export { ENABLE_RAIL_GRAPH } from "./types";
export { buildRailGraphInputFromGitLog } from "./build-input";
export { allocateLanes } from "./lane-allocator";
export { normalizeRefs } from "./refs-normalize";
export { branchNameToColorKey } from "./color-mapper";
export { RailGraphCanvas } from "./RailGraphCanvas";
export type { RailGraphCanvasProps } from "./RailGraphCanvas";
export {
  buildRailRowMetrics,
  computeVisibleRange,
  computeVisibleRangeFromMetrics,
  filterRailGeometryToVisibleRange,
} from "./RailGraphVirtualizer";
export type { RailRowMetrics, RailVisibleRange } from "./RailGraphVirtualizer";
export {
  createRailFrameScheduler,
  createRailPerformanceSampler,
} from "./raf-scheduler";
export type {
  RailFrameScheduler,
  RailFrameSchedulerHost,
  RailPerformanceSampler,
} from "./raf-scheduler";
