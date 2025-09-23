import type { AssignmentConfig } from "@/types/modules/assignments/config";

export const DEFAULT_ASSIGNMENT_CONFIG: AssignmentConfig = {
  execution: {
    timeout_secs: 10,
    max_memory: 8_589_934_592, // 8 GiB
    max_cpus: 2,
    max_uncompressed_size: 100_000_000, // 100 MB
    max_processes: 256,
  },
  marking: {
    marking_scheme: "exact",
    feedback_scheme: "auto",
    deliminator: "&-=-&",
    grading_policy: "best",
    limit_attempts: true,
    max_attempts: 10,
    pass_mark: 50,
    allow_practice_submissions: false,
    dissalowed_code: [],
  },
  project: {
    language: "cpp",
    submission_mode: "manual",
  },
  output: {
    stdout: true,
    stderr: false,
    retcode: false,
  },
  gatlam: {
    population_size: 50,
    number_of_generations: 100,
    selection_size: 10,
    reproduction_probability: 0.5,
    crossover_probability: 0.7,
    mutation_probability: 0.01,
    crossover_type: "onepoint",
    mutation_type: "bitflip",
    genes: [
      { min_value: 0, max_value: 1 },
    ],
    omega1: 0.3,
    omega2: 0.2,
    omega3: 0.5,
    task_spec: {
      max_runtime_ms: 5000,
      valid_return_codes: [0],
      forbidden_outputs: [],
    },
    max_parallel_chromosomes: 4,
    verbose: false,
  },
  security: {
  password_enabled: false,
  password_pin: null,
  cookie_ttl_minutes: 120,
  bind_cookie_to_user: true,
  allowed_cidrs: []
  },
  code_coverage: {
    code_coverage_weight: 10.0,
  }
};
