// Base components
export { Container } from "./components/Container";
export type { ContainerProps } from "./components/Container";

export { Title } from "./components/Title";
export type { TitleProps } from "./components/Title";

export { Sequence } from "./components/Sequence";
export type { SequenceProps } from "./components/Sequence";

export { Metric } from "./components/Metric";
export type { MetricProps } from "./components/Metric";

// Frame rendering
export { MoronFrame } from "./MoronFrame";
export type { MoronFrameProps } from "./MoronFrame";

// Frame state types (Rust FrameState JSON contract)
export type {
  FrameState,
  ElementState,
  ElementKind,
  ThemeState,
} from "./types";

// Templates (empty during scaffold phase)
// Re-exports will be added here as templates are implemented.
