import type { Timestamp } from "@/types/common";

export interface InterpreterInfo extends Timestamp {
  id: number;
  assignment_id: number;
  filename: string;
  path: string;
  command: string;
}