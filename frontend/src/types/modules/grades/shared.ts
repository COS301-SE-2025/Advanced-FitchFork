import type { Timestamp } from "@/types/common";


export interface Grade extends Timestamp {
  id: number;
  assignment_id: number;
  student_id: number;
  submission_id: number;
  score: number;
}