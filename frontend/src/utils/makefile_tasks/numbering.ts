import { listTasks } from '@/services/modules/assignments/tasks';

export async function getNextTaskNumber(
  moduleId: number,
  assignmentId: number
): Promise<number> {
  const res = await listTasks(moduleId, assignmentId);
  if (!res.success || !Array.isArray(res.data) || res.data.length === 0) return 1;

  const maxNum = Math.max(...res.data.map((t: any) => Number(t.task_number) || 0));
  return Number.isFinite(maxNum) ? maxNum + 1 : 1;
}
