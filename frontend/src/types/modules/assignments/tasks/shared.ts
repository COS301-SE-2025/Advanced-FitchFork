//   id: number;                    // auto-incrementing primary key
//   assignment_id: number;         // foreign key to assignments.id
//   task_number: number;           // identifies the task's order or ID within the assignment
//   command: string;  

import type { Timestamp } from "@/types/common";

export interface Task extends Timestamp {
    id: number;
    assignment_id: number;
    task_number: number;
    command: string;
}