/**
 * ---- Enumerations (serialized values must match Rust/serde "lowercase") ----
 */

/** Marking schemes */
export const MARKING_SCHEMES = ['exact', 'percentage', 'regex'] as const;
/** Feedback schemes */
export const FEEDBACK_SCHEMES = ['auto', 'manual', 'ai'] as const;
/** Languages (Rust currently supports only C++ and Java) */
export const LANGUAGES = ['c', 'cpp', 'java', 'python', 'go', 'rust'] as const;

/**
 * Submission modes
 * NOTE: backend treats anything not 'manual' as the asynchronous/AI pipeline.
 * Keep legacy values for compatibility.
 */
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

export const LANGUAGE_LABELS: Record<(typeof LANGUAGES)[number], string> = {
  c: 'C',
  cpp: 'C++',
  java: 'Java',
  python: 'Python',
  go: 'GoLang',
  rust: 'Rust',
};
export const LANGUAGE_OPTIONS = LANGUAGES.map((val) => ({
  label: LANGUAGE_LABELS[val],
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

/** Late submission policy (new in ExecutionConfig.marking.late). */
export interface LateOptions {
  /** Allow late submissions at all. */
  allow_late_submissions: boolean;
  /**
   * Minutes after due date during which late submissions are still accepted.
   * (If `allow_late_submissions` is true.)
   */
  late_window_minutes: number;
  /**
   * Cap for the earned mark when late, expressed as a percentage of total (0–100).
   * e.g. 60 means “late submissions can earn at most 60% of total”.
   */
  late_max_percent: number;
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
   * If false, practice uploads are rejected for students (staff unaffected).
   */
  allow_practice_submissions: boolean;

  /** Substrings to flag as disallowed in source files (serialized as `dissalowed_code`). */
  dissalowed_code: string[];

  /** late submission policy. */
  late: LateOptions;
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

/** Security options (SecurityOptions in Rust). */
export interface AssignmentSecurityConfig {
  /** If true, students must unlock the assignment. */
  password_enabled: boolean;
  /** Optional PIN in plain text; null/undefined means no PIN set. */
  password_pin?: string | null;
  /** Minutes the unlock cookie stays valid. */
  cookie_ttl_minutes: number;
  /** If true, bind cookie to user id (harder to share). */
  bind_cookie_to_user: boolean;
  /** Optional CIDR allowlist; empty => allow all. */
  allowed_cidrs: string[];
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

export interface CodeCoverage {
  code_coverage_weight: number;
}

/**
 * Top-level assignment configuration (ExecutionConfig in Rust).
 */
export interface AssignmentConfig {
  execution: AssignmentExecutionConfig;
  marking: AssignmentMarkingConfig;     // ← includes .late now
  project: AssignmentProjectConfig;
  output: AssignmentOutputConfig;
  gatlam: GatlamConfig;
  security: AssignmentSecurityConfig;
  code_coverage: CodeCoverage;
}
