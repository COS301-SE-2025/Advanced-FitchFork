import { useCallback, useEffect, useMemo, useState } from 'react';
import { useParams, Outlet } from 'react-router-dom';
import { Spin } from 'antd';

import { useModule } from '@/context/ModuleContext';
import { AssignmentProvider } from '@/context/AssignmentContext';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';

import { getAssignmentDetails, getAssignmentReadiness } from '@/services/modules/assignments';
import { getMemoOutput } from '@/services/modules/assignments/memo-output';
import { getMarkAllocator } from '@/services/modules/assignments/mark-allocator';
import {
  getAssignmentConfig,
  setAssignmentConfig,
  resetAssignmentConfig, // ← NEW import
} from '@/services/modules/assignments/config';

import type { Assignment, AssignmentFile, AssignmentReadiness } from '@/types/modules/assignments';
import type { MemoTaskOutput } from '@/types/modules/assignments/memo-output';
import type { MarkAllocatorFile } from '@/types/modules/assignments/mark-allocator';
import type { AssignmentConfig } from '@/types/modules/assignments/config';

interface AssignmentDetails extends Assignment {
  files: AssignmentFile[];
}

const mergeConfig = (
  base: AssignmentConfig,
  patch: Partial<AssignmentConfig>,
): AssignmentConfig => ({
  ...base,
  ...(patch as Partial<AssignmentConfig>),
  execution: { ...base.execution, ...(patch.execution ?? {}) },
  marking: { ...base.marking, ...(patch.marking ?? {}) },
  project: { ...base.project, ...(patch.project ?? {}) },
  output: { ...base.output, ...(patch.output ?? {}) },
  gatlam: { ...base.gatlam, ...(patch.gatlam ?? {}) },
});

export default function WithAssignmentContext() {
  const module = useModule();
  const { assignment_id } = useParams();
  const assignmentIdNum = Number(assignment_id);
  const { setBreadcrumbLabel } = useBreadcrumbContext();

  const [assignment, setAssignment] = useState<AssignmentDetails | null>(null);
  const [memoOutput, setMemoOutput] = useState<MemoTaskOutput[]>([]);
  const [markAllocator, setMarkAllocator] = useState<MarkAllocatorFile | null>(null);
  const [readiness, setReadiness] = useState<AssignmentReadiness | null>(null);
  const [config, setConfig] = useState<AssignmentConfig | null>(null);
  const [loading, setLoading] = useState(true);

  const refreshAssignment = useCallback(async () => {
    setLoading(true);
    try {
      const [detailsRes, readinessRes, memoRes, allocatorRes, configRes] = await Promise.all([
        getAssignmentDetails(module.id, assignmentIdNum),
        getAssignmentReadiness(module.id, assignmentIdNum),
        getMemoOutput(module.id, assignmentIdNum),
        getMarkAllocator(module.id, assignmentIdNum),
        getAssignmentConfig(module.id, assignmentIdNum),
      ]);

      if (detailsRes.success && detailsRes.data) {
        const details = detailsRes.data;
        setAssignment(details);
        setBreadcrumbLabel(`modules/${module.id}/assignments/${details.id}`, details.name);
      }

      if (readinessRes.success) setReadiness(readinessRes.data);
      if (memoRes.success && memoRes.data) setMemoOutput(memoRes.data);
      if (allocatorRes.success && allocatorRes.data) setMarkAllocator(allocatorRes.data);
      setConfig(configRes.success ? (configRes.data ?? null) : null);
    } finally {
      setLoading(false);
    }
  }, [assignmentIdNum, module.id]);

  useEffect(() => {
    if (!Number.isNaN(assignmentIdNum)) {
      void refreshAssignment();
    } else {
      setLoading(false);
    }
  }, [assignmentIdNum, module.id, refreshAssignment]);

  const updateConfig = useCallback(
    async (patch: Partial<AssignmentConfig>) => {
      if (!assignment) return;
      if (!config) throw new Error('No current config loaded; cannot apply patch.');

      const next: AssignmentConfig = mergeConfig(config, patch);
      const prev = config;
      setConfig(next); // optimistic

      try {
        const res = await setAssignmentConfig(module.id, assignment.id, next);
        if (!res.success) throw new Error(res.message ?? 'Failed to save config');
        if (res.data) setConfig(res.data); // keep server canonical if provided
      } catch (e) {
        setConfig(prev ?? null); // rollback
        throw e;
      }
    },
    [assignment, config, module.id],
  );

  // NEW: reset to server defaults (POST /config/reset)
  const resetConfig = useCallback(async () => {
    if (!assignment) return;
    const prev = config;

    try {
      const res = await resetAssignmentConfig(module.id, assignment.id);
      if (!res.success || !res.data) {
        throw new Error(res.message ?? 'Failed to reset config');
      }
      setConfig(res.data); // server returns the default it saved
    } catch (e) {
      // keep previous config if reset failed
      setConfig(prev ?? null);
      throw e;
    }
  }, [assignment, module.id, config]);

  const value = useMemo(
    () => ({
      assignment: assignment!, // safe due to loading gate
      memoOutput,
      markAllocator,
      readiness,
      config,
      loading,
      refreshAssignment,
      updateConfig,
      resetConfig, // ← expose reset
    }),
    [
      assignment,
      memoOutput,
      markAllocator,
      readiness,
      config,
      loading,
      refreshAssignment,
      updateConfig,
      resetConfig,
    ],
  );

  if (loading || !assignment) return <Spin className="p-6" tip="Loading assignment..." />;

  return (
    <AssignmentProvider value={value}>
      <Outlet />
    </AssignmentProvider>
  );
}
