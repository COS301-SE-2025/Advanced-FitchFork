import { createTask } from '@/services/modules/assignments/tasks';
import { getNextTaskNumber } from './numbering';

export async function createTasksFromMakefileTargets(
  moduleId: number,
  assignmentId: number,
  targets: string[],
  refreshAssignment?: () => Promise<void>
) : Promise<number> {
  if (targets.length === 0) return 0;

  let next = await getNextTaskNumber(moduleId, assignmentId);
  let created = 0;

  for (let i = 0; i < targets.length; i++) {
    const target = targets[i];
    const name = `Task ${i + 1}`;
    const command = `make ${target}`;

    const res = await createTask(moduleId, assignmentId, {
      task_number: next++,
      name,
      command,
      code_coverage: false,
    });

    if (res.success) created++;
  }

  if (created > 0) {
    await refreshAssignment?.();
  }

  return created;
}
