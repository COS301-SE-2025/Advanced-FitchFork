export interface ExecutionConfig {
  timeout_secs: number;
  max_memory: string;
  max_cpus: string;
  max_uncompressed_size: number;
  max_processes: number;
  language: string;
}