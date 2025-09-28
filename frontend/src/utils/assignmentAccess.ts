const key = (moduleId: number, assignmentId: number) =>
  `assignment_pin:${moduleId}:${assignmentId}`;

export function getAssignmentPin(moduleId: number, assignmentId: number): string | null {
  return sessionStorage.getItem(key(moduleId, assignmentId));
}

export function setAssignmentPin(moduleId: number, assignmentId: number, pin: string) {
  sessionStorage.setItem(key(moduleId, assignmentId), pin);
}

export function clearAssignmentPin(moduleId: number, assignmentId: number) {
  sessionStorage.removeItem(key(moduleId, assignmentId));
}
