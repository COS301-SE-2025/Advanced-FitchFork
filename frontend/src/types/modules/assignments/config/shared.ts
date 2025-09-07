/**
 * ---- Enumerations (serialized values must match Rust/serde "lowercase") ----
 */

/** Marking schemes */
export const MARKING_SCHEMES = ['exact', 'percentage', 'regex'] as const;
/** Feedback schemes */
export const FEEDBACK_SCHEMES = ['auto', 'manual', 'ai'] as const;
/** Languages (Rust currently supports only C++ and Java) */
export const LANGUAGES = ['cpp', 'java'] as const;
/** Submission modes */
export const SUBMISSION_MODES = ['manual', 'gatlam', 'rng', 'codecoverage'] as const;
/** Grading policies (mirrors Rust enum) */
export const GRADING_POLICIES = ['best', 'last'] as const;

/** GA: crossover & mutation types */
export const CROSSOVER_TYPES = ['onepoint', 'twopoint', 'uniform'] as const;
export const MUTATION_TYPES = ['bitflip', 'swap', 'scramble'] as const;

/** Select options */
export const MARKING_SCHEME_OPTIONS = MARKING_SCHEMES.map((val) => ({
  label: val.charAt(0).toUpperCase() + val.slice(1),
  value: val,
}));

export const FEEDBACK_SCHEME_OPTIONS = FEEDBACK_SCHEMES.map((val) => ({
  label: val.charAt(0).toUpperCase() + val.slice(1),
  value: val,
}));

export const LANGUAGE_OPTIONS = LANGUAGES.map((val) => ({
  label: val.charAt(0).toUpperCase() + val.slice(1),
  value: val,
}));

export const SUBMISSION_MODE_OPTIONS = SUBMISSION_MODES.map((val) => ({
  label: val.charAt(0).toUpperCase() + val.slice(1),
  value: val,
}));

export const GRADING_POLICY_OPTIONS = GRADING_POLICIES.map((val) => ({
  label: val.charAt(0).toUpperCase() + val.slice(1),
  value: val,
}));

export const CROSSOVER_TYPE_OPTIONS = CROSSOVER_TYPES.map((val) => ({
  label: val.charAt(0).toUpperCase() + val.slice(1),
  value: val,
}));

export const MUTATION_TYPE_OPTIONS = MUTATION_TYPES.map((val) => ({
  label: val.charAt(0).toUpperCase() + val.slice(1),
  value: val,
}));

/**
 * ---- Type unions from const arrays ----
 */
export type MarkingScheme = (typeof MARKING_SCHEMES)[number];
export type FeedbackScheme = (typeof FEEDBACK_SCHEMES)[number];
export type Language = (typeof LANGUAGES)[number];
export type SubmissionMode = (typeof SUBMISSION_MODES)[number];
export type GradingPolicy = (typeof GRADING_POLICIES)[number];
export type CrossoverType = (typeof CROSSOVER_TYPES)[number];
export type MutationType = (typeof MUTATION_TYPES)[number];

/**
 * ---- Top-level config sections (mirrors Rust structs) ----
 */

/** Project setup options. */
export interface AssignmentProjectConfig {
  /** Language used for the project (cpp, java). */
  language: Language;
  /** How submissions are generated (manual, gatlam, rng, codecoverage). */
  submission_mode: SubmissionMode;
}

/** Resource constraints for running a submission (ExecutionLimits). */
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

/** Configuration for marking and feedback generation (MarkingOptions). */
export interface AssignmentMarkingConfig {
  /** Strategy used to mark student submissions. */
  marking_scheme: MarkingScheme;

  /** Method used to generate feedback for the submission. */
  feedback_scheme: FeedbackScheme;

  /** Policy for selecting final grade across submissions. */
  grading_policy: GradingPolicy;

  /**
   * String delimiter used for splitting output sections.
   * NOTE: spelling matches backend field (`deliminator`) for compatibility.
   */
  deliminator: string;

  /** Maximum number of attempts (only enforced if `limit_attempts` is true). */
  max_attempts: number;

  /** If false, attempt limits are not enforced. */
  limit_attempts: boolean;

  /** Minimum percentage required to pass (0–100). */
  pass_mark: number;

  /**
   * If true, **students** may make practice submissions.
   * If false (default), practice uploads are rejected for students.
   * Staff (lecturer/assistant/admin) are unaffected (always allowed).
   */
  allow_practice_submissions: boolean;
}

/** Options for execution output capture (ExecutionOutputOptions). */
export interface AssignmentOutputConfig {
  /** Whether to capture stdout. */
  stdout: boolean;
  /** Whether to capture stderr. */
  stderr: boolean;
  /** Whether to include return code. */
  retcode: boolean;
}

/**
 * ---- GATLAM-related config (mirrors Rust GATLAM & TaskSpecConfig) ----
 */
export interface GeneConfig {
  min_value: number;
  max_value: number;
}

export interface TaskSpecConfig {
  /** Return codes that are considered success. */
  valid_return_codes: number[];
  /** Optional hard runtime cap (milliseconds). */
  max_runtime_ms?: number;
  /** Disallowed substrings in outputs. */
  forbidden_outputs: string[];
}

export interface GatlamConfig {
  // ---- GA Config ----
  population_size: number;
  number_of_generations: number;
  selection_size: number;
  reproduction_probability: number;
  crossover_probability: number;
  mutation_probability: number;
  genes: GeneConfig[];
  crossover_type: CrossoverType;
  mutation_type: MutationType;

  // ---- Components ----
  omega1: number;
  omega2: number;
  omega3: number;

  // ---- TaskSpec ----
  task_spec: TaskSpecConfig;

  // ---- Optional runtime flags ----
  max_parallel_chromosomes: number;
  verbose: boolean;
}

/**
 * Top-level assignment configuration (ExecutionConfig in Rust).
 * Combines execution limits, marking rules, project setup, output options, and GA/TLAM config.
 */
export interface AssignmentConfig {
  execution: AssignmentExecutionConfig;
  marking: AssignmentMarkingConfig;
  project: AssignmentProjectConfig;
  output: AssignmentOutputConfig;
  gatlam: GatlamConfig;
}
