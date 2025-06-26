import type { Timestamp } from "@/types/common";

export interface Task extends Timestamp {
    id: number;
    assignment_id: number;
    task_number: number;
    command: string;
}