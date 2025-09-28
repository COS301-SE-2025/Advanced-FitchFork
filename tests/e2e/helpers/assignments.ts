import type { APIRequestContext } from '@playwright/test';
import { login } from '@helpers/auth';

type AssignmentType = 'assignment' | 'practical';
type AssignmentStatus = 'setup' | 'ready' | 'open' | 'closed' | 'archived';

export type AssignmentSeedInput = {
  name: string;
  description?: string;
  assignment_type?: AssignmentType;
  available_from?: string;
  due_date?: string;
};

export type AssignmentRecord = {
  id: number;
  module_id: number;
  name: string;
  description: string;
  assignment_type: AssignmentType;
  available_from: string;
  due_date: string;
  status: AssignmentStatus;
  created_at?: string;
  updated_at?: string;
};

type ApiEnvelope<T> = {
  success: boolean;
  data?: T;
  message?: string;
};

function defaultAssignmentPayload(input: AssignmentSeedInput) {
  const now = new Date();
  const available = input.available_from ?? now.toISOString();
  const dueDate = input.due_date ?? new Date(now.getTime() + 7 * 24 * 60 * 60 * 1000).toISOString();

  return {
    name: input.name,
    description: input.description ?? '',
    assignment_type: input.assignment_type ?? 'assignment',
    available_from: available,
    due_date: dueDate,
  };
}

export async function createAssignment(
  api: APIRequestContext,
  moduleId: number,
  input: AssignmentSeedInput,
  token: string,
): Promise<AssignmentRecord> {
  const payload = defaultAssignmentPayload(input);
  const res = await api.post(`/api/modules/${moduleId}/assignments`, {
    data: payload,
    headers: { Authorization: `Bearer ${token}` },
  });

  if (!res.ok()) {
    throw new Error(`createAssignment failed ${res.status()}: ${await res.text()}`);
  }

  const json = (await res.json()) as ApiEnvelope<AssignmentRecord>;
  if (!json.success || !json.data) {
    throw new Error(`createAssignment error: ${json.message ?? 'unknown error'}`);
  }

  return json.data;
}

export async function deleteAssignment(
  api: APIRequestContext,
  moduleId: number,
  assignmentId: number,
  token: string,
): Promise<void> {
  const res = await api.delete(`/api/modules/${moduleId}/assignments/${assignmentId}`, {
    headers: { Authorization: `Bearer ${token}` },
  });

  if (!res.ok()) {
    throw new Error(`deleteAssignment(${assignmentId}) failed ${res.status()}: ${await res.text()}`);
  }

  const json = (await res.json()) as ApiEnvelope<null>;
  if (!json.success) {
    throw new Error(`deleteAssignment(${assignmentId}) error: ${json.message ?? 'unknown error'}`);
  }
}

export async function listAssignments(
  api: APIRequestContext,
  moduleId: number,
  token: string,
  params: { page?: number; per_page?: number } = {},
): Promise<{ assignments: AssignmentRecord[]; total: number }> {
  const search = new URLSearchParams({
    page: String(params.page ?? 1),
    per_page: String(params.per_page ?? 50),
  });

  const res = await api.get(`/api/modules/${moduleId}/assignments?${search.toString()}`, {
    headers: { Authorization: `Bearer ${token}` },
  });

  if (!res.ok()) {
    throw new Error(`listAssignments failed ${res.status()}: ${await res.text()}`);
  }

  const json = (await res.json()) as ApiEnvelope<{
    assignments: AssignmentRecord[];
    total: number;
  }>;

  if (!json.success || !json.data) {
    throw new Error(`listAssignments error: ${json.message ?? 'unknown error'}`);
  }

  return {
    assignments: json.data.assignments ?? [],
    total: json.data.total ?? json.data.assignments?.length ?? 0,
  };
}

export async function bulkDeleteAssignments(
  api: APIRequestContext,
  moduleId: number,
  assignmentIds: number[],
  token: string,
): Promise<void> {
  if (assignmentIds.length === 0) return;

  const res = await api.delete(`/api/modules/${moduleId}/assignments/bulk`, {
    data: { assignment_ids: assignmentIds },
    headers: { Authorization: `Bearer ${token}` },
  });

  if (!res.ok()) {
    throw new Error(`bulkDeleteAssignments failed ${res.status()}: ${await res.text()}`);
  }

  const json = (await res.json()) as ApiEnvelope<{ deleted: number }>;
  if (!json.success) {
    throw new Error(`bulkDeleteAssignments error: ${json.message ?? 'unknown error'}`);
  }
}

export async function createAssignmentAsAdmin(
  api: APIRequestContext,
  moduleId: number,
  input: AssignmentSeedInput,
  adminUser = 'admin',
  adminPassword = process.env.E2E_TEST_USER_PASSWORD ?? '1',
): Promise<AssignmentRecord> {
  const { data } = await login(api, adminUser, adminPassword);
  if (!data?.token) throw new Error('Failed to obtain admin token');
  return createAssignment(api, moduleId, input, data.token);
}

export async function deleteAssignmentAsAdmin(
  api: APIRequestContext,
  moduleId: number,
  assignmentId: number,
  adminUser = 'admin',
  adminPassword = process.env.E2E_TEST_USER_PASSWORD ?? '1',
): Promise<void> {
  const { data } = await login(api, adminUser, adminPassword);
  if (!data?.token) throw new Error('Failed to obtain admin token');
  await deleteAssignment(api, moduleId, assignmentId, data.token);
}

export async function purgeAssignmentsAsAdmin(
  api: APIRequestContext,
  moduleId: number,
  adminUser = 'admin',
  adminPassword = process.env.E2E_TEST_USER_PASSWORD ?? '1',
): Promise<void> {
  const { data } = await login(api, adminUser, adminPassword);
  if (!data?.token) throw new Error('Failed to obtain admin token');
  const token = data.token;
  const { assignments } = await listAssignments(api, moduleId, token, { per_page: 100 });
  if (!assignments.length) return;
  await bulkDeleteAssignments(api, moduleId, assignments.map((a) => a.id), token);
}
