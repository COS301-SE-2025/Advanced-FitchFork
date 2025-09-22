/**
 * ---- Enumerations (serialized values must match Rust/serde "lowercase") ----
 */

/** Marking schemes */
export const MARKING_SCHEMES = ['exact', 'percentage', 'regex'] as const;
/** Feedback schemes */
export const FEEDBACK_SCHEMES = ['auto', 'manual', 'ai'] as const;
/** Languages (Rust currently supports only C++ and Java) */
export const LANGUAGES = [
  'c',
  'cpp',
  'java',
  'ml',
  'pascal',
  'ada',
  'lisp',
  'scheme',
  'haskell',
  'fortran',
  'ascii',
  'vhdl',
  'perl',
  'matlab',
  'python',
  'mips',
  'prolog',
  'spice',
  'vb',
  'csharp',
  'modula2',
  'a8086',
  'javascript',
  'plsql',
] as const;

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

// pretty labels (C++/C#/PL/SQL/…)
const NOT_IMPL = ' (not implemented)';

export const LANGUAGE_LABELS: Record<(typeof LANGUAGES)[number], string> = {
  c: `C${NOT_IMPL}`,
  cpp: 'C++',
  java: 'Java',
  ml: `ML${NOT_IMPL}`,
  pascal: `Pascal${NOT_IMPL}`,
  ada: `Ada${NOT_IMPL}`,
  lisp: `Lisp${NOT_IMPL}`,
  scheme: `Scheme${NOT_IMPL}`,
  haskell: `Haskell${NOT_IMPL}`,
  fortran: `Fortran${NOT_IMPL}`,
  ascii: `ASCII${NOT_IMPL}`,
  vhdl: `VHDL${NOT_IMPL}`,
  perl: `Perl${NOT_IMPL}`,
  matlab: `MATLAB${NOT_IMPL}`,
  python: `Python${NOT_IMPL}`,
  mips: `MIPS${NOT_IMPL}`,
  prolog: `Prolog${NOT_IMPL}`,
  spice: `SPICE${NOT_IMPL}`,
  vb: `VB${NOT_IMPL}`,
  csharp: `C#${NOT_IMPL}`,
  modula2: `Modula-2${NOT_IMPL}`,
  a8086: `8086 Assembly${NOT_IMPL}`,
  javascript: `JavaScript${NOT_IMPL}`,
  plsql: `PL/SQL${NOT_IMPL}`,
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

  /** Substrings to flag as disallowed in source files (serialized as `dissalowed_code`). */
  dissalowed_code: string[];
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
  code_coverage_required: number;
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
  security: AssignmentSecurityConfig;
  code_coverage: CodeCoverage;
}
