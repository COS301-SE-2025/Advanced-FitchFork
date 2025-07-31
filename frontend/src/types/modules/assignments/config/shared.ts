/**
 * List of available marking schemes.
 * 
 * - `exact`: Requires exact output match.
 * - `percentage`: Partial credit based on output similarity.
 * - `regex`: Output is matched against a regular expression.
 */
export const MARKING_SCHEMES = ['exact', 'percentage', 'regex'] as const;

/**
 * List of available feedback generation schemes.
 * 
 * - `auto`: Automatically generated feedback (e.g. simple diffs).
 * - `manual`: Requires manual grader input.
 * - `ai`: Uses AI-based feedback generation.
 */
export const FEEDBACK_SCHEMES = ['auto', 'manual', 'ai'] as const;

/**
 * Ant Design `<Select />` options for marking schemes.
 * Each option is in the form `{ label, value }`, both set to the scheme name.
 */
export const MARKING_SCHEME_OPTIONS = MARKING_SCHEMES.map((val) => ({
  label: val.charAt(0).toUpperCase() + val.slice(1),
  value: val,
}));

/**
 * Ant Design `<Select />` options for feedback schemes.
 * Each option is in the form `{ label, value }`, both set to the scheme name.
 */
export const FEEDBACK_SCHEME_OPTIONS = FEEDBACK_SCHEMES.map((val) => ({
  label: val.charAt(0).toUpperCase() + val.slice(1),
  value: val,
}));

/**
 * Type union of all supported marking schemes.
 * Derived from the `MARKING_SCHEMES` array.
 */
export type MarkingScheme = typeof MARKING_SCHEMES[number];

/**
 * Type union of all supported feedback schemes.
 * Derived from the `FEEDBACK_SCHEMES` array.
 */
export type FeedbackScheme = typeof FEEDBACK_SCHEMES[number];

/**
 * Resource constraints for running an assignment submission.
 */
export interface AssignmentExecutionConfig {
  /** Time limit for execution (in seconds). */
  timeout_secs: number;

  /** Max memory usage allowed (in bytes). */
  max_memory: number;

  /** Max number of CPU cores allowed. */
  max_cpus: number;

  /** Max uncompressed submission size (in bytes). */
  max_uncompressed_size: number;

  /** Max number of processes allowed. */
  max_processes: number;
}

/**
 * Configuration for how assignment outputs are marked and how feedback is generated.
 */
export interface AssignmentMarkingConfig {
  /** Strategy used to mark student submissions. */
  marking_scheme: MarkingScheme;

  /** Method used to generate feedback for the submission. */
  feedback_scheme: FeedbackScheme;

  /** String delimiter used for splitting output sections. */
  deliminator: string;
}

/**
 * Top-level assignment configuration.
 * Combines execution limits and marking rules.
 */
export interface AssignmentConfig {
  execution: AssignmentExecutionConfig;
  marking: AssignmentMarkingConfig;
}
