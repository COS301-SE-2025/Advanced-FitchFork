import { useCallback, useEffect, useMemo, useState } from 'react';
import { useParams, Outlet, useNavigate } from 'react-router-dom';

import AssignmentLoadingPlaceholder from '@/components/providers/AssignmentLoadingPlaceholder';

import { useModule } from '@/context/ModuleContext';
import { AssignmentProvider } from '@/context/AssignmentContext';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';

import {
  getAssignmentDetails,
  getAssignmentReadiness,
  getAssignmentStats,
} from '@/services/modules/assignments';
import { getMemoOutput } from '@/services/modules/assignments/memo-output';
import { getMarkAllocator } from '@/services/modules/assignments/mark-allocator';
import {
  getAssignmentConfig,
  setAssignmentConfig,
  resetAssignmentConfig,
} from '@/services/modules/assignments/config';

import type {
  Assignment,
  AssignmentFile,
  AssignmentReadiness,
  AssignmentStats,
  AttemptsInfo,
  BestMark,
  AssignmentPolicy,
} from '@/types/modules/assignments';
import type { MemoTaskOutput } from '@/types/modules/assignments/memo-output';
import type { MarkAllocatorFile } from '@/types/modules/assignments/mark-allocator';
import type { AssignmentConfig } from '@/types/modules/assignments/config';
import { useAuth } from '@/context/AuthContext';

const mergeConfig = (
  base: AssignmentConfig,
  patch: Partial<AssignmentConfig>,
): AssignmentConfig => ({
  ...base,
  ...(patch as Partial<AssignmentConfig>),
  execution: { ...base.execution, ...(patch.execution ?? {}) },
  marking: { ...base.marking, ...(patch.marking ?? {}) },
  project: { ...base.project, ...(patch.project ?? {}) },
  gatlam: { ...base.gatlam, ...(patch.gatlam ?? {}) },
});

export default function WithAssignmentContext() {
  const module = useModule();
  const { assignment_id } = useParams();
  const auth = useAuth();
  const navigate = useNavigate();
  const assignmentIdNum = Number(assignment_id);
  const { setBreadcrumbLabel } = useBreadcrumbContext();

  const [assignment, setAssignment] = useState<Assignment | null>(null);
  const [assignmentFiles, setAssignmentFiles] = useState<AssignmentFile[]>([]);
  const [attempts, setAttempts] = useState<AttemptsInfo | null>(null);
  const [bestMark, setBestMark] = useState<BestMark | null>(null);
  const [assignmentStats, setAssignmentStats] = useState<AssignmentStats | null>(null);

  const [memoOutput, setMemoOutput] = useState<MemoTaskOutput[]>([]);
  const [markAllocator, setMarkAllocator] = useState<MarkAllocatorFile | null>(null);
  const [readiness, setReadiness] = useState<AssignmentReadiness | null>(null);
  const [policy, setPolicy] = useState<AssignmentPolicy | null>(null);
  const [config, setConfig] = useState<AssignmentConfig | null>(null);

  const [loading, setLoading] = useState(true);

  const isLecturerLike =
    auth.isLecturer(module.id) || auth.isAssistantLecturer(module.id) || auth.isAdmin;
  const isStaffLike = isLecturerLike || auth.isTutor(module.id);

  const refreshAssignmentStats = useCallback(async () => {
    if (!isStaffLike || Number.isNaN(assignmentIdNum)) {
      setAssignmentStats(null);
      return;
    }
    try {
      const statsRes = await getAssignmentStats(module.id, assignmentIdNum);
      setAssignmentStats(statsRes.success ? (statsRes.data ?? null) : null);
    } catch {
      setAssignmentStats(null);
    }
  }, [assignmentIdNum, isStaffLike, module.id]);

  const refreshAssignment = useCallback(async () => {
    setLoading(true);
    try {
      if (isLecturerLike) {
        // lecturer / assistant lecturer / admin → full details
        const [detailsRes, readinessRes, memoRes, allocatorRes, configRes] = await Promise.all([
          getAssignmentDetails(module.id, assignmentIdNum),
          getAssignmentReadiness(module.id, assignmentIdNum),
          getMemoOutput(module.id, assignmentIdNum),
          getMarkAllocator(module.id, assignmentIdNum),
          getAssignmentConfig(module.id, assignmentIdNum),
        ]);

        if (detailsRes.success && detailsRes.data) {
          const details = detailsRes.data;
          setAssignment(details.assignment);
          setAssignmentFiles(details.files);
          setAttempts(details.attempts ?? null);
          setBestMark(details.best_mark ?? null);
          setPolicy(details.policy ?? null);

          setBreadcrumbLabel(
            `modules/${module.id}/assignments/${details.assignment.id}`,
            details.assignment.name,
          );
        }

        if (readinessRes.success) setReadiness(readinessRes.data);
        if (memoRes.success && memoRes.data) setMemoOutput(memoRes.data);
        if (allocatorRes.success && allocatorRes.data) setMarkAllocator(allocatorRes.data);
        setConfig(configRes.success ? (configRes.data ?? null) : null);
      } else {
        // students / tutors → safe subset
        const [detailsRes, readinessRes] = await Promise.all([
          getAssignmentDetails(module.id, assignmentIdNum),
          getAssignmentReadiness(module.id, assignmentIdNum),
        ]);

        if (!detailsRes.success) {
          const msg = (detailsRes.message || '').toLowerCase();

          // hard network/IP block → go to access denied page
          if (msg.includes('ip') || msg.includes('not allowed') || msg.includes('forbidden')) {
            navigate(`/modules/${module.id}/assignments/${assignmentIdNum}/access-denied`, {
              replace: true,
              state: { message: detailsRes.message },
            });
            return;
          }

          // password required/invalid → go to verify page
          if (msg.includes('password') || msg.includes('pin')) {
            const next = encodeURIComponent(window.location.pathname);
            navigate(`/modules/${module.id}/assignments/${assignmentIdNum}/verify?next=${next}`, {
              replace: true,
            });
            return;
          }

          // default fallback
          navigate('/forbidden', { replace: true });
          return;
        }

        if (detailsRes.success && detailsRes.data) {
          const details = detailsRes.data;
          setAssignment(details.assignment);
          setAssignmentFiles(details.files);
          setAttempts(details.attempts ?? null);
          setBestMark(details.best_mark ?? null);
          setPolicy(details.policy ?? null);

          setBreadcrumbLabel(
            `modules/${module.id}/assignments/${details.assignment.id}`,
            details.assignment.name,
          );
        }

        if (readinessRes.success) setReadiness(readinessRes.data);
        setMemoOutput([]);
        setMarkAllocator(null);
        setConfig(null);
      }

      await refreshAssignmentStats();
    } finally {
      setLoading(false);
    }
  }, [assignmentIdNum, module.id, isLecturerLike, refreshAssignmentStats]);

  useEffect(() => {
    if (!Number.isNaN(assignmentIdNum)) {
      void refreshAssignment();
    } else {
      setLoading(false);
    }
  }, [assignmentIdNum, module.id, refreshAssignment]);

  const incrementAttempts = useCallback(() => {
    setAttempts((prev) => {
      if (!prev) return prev;
      const used = prev.used + 1;

      let remaining = prev.remaining;
      if (prev.limit_attempts && prev.max != null) {
        remaining = Math.max(prev.max - used, 0);
      }

      return { ...prev, used, remaining };
    });
  }, []);

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
        if (res.data) setConfig(res.data);
      } catch (e) {
        setConfig(prev ?? null); // rollback
        throw e;
      }
    },
    [assignment, config, module.id],
  );

  const resetConfig = useCallback(async () => {
    if (!assignment) return;
    const prev = config;

    try {
      const res = await resetAssignmentConfig(module.id, assignment.id);
      if (!res.success || !res.data) {
        throw new Error(res.message ?? 'Failed to reset config');
      }
      setConfig(res.data);
    } catch (e) {
      setConfig(prev ?? null);
      throw e;
    }
  }, [assignment, module.id, config]);

  const value = useMemo(
    () => ({
      assignment: assignment!, // safe when not blocked
      assignmentFiles,
      attempts,
      bestMark,
      memoOutput,
      markAllocator,
      readiness,
      policy,
      config,
      loading,
      assignmentStats,
      refreshAssignmentStats,
      refreshAssignment,
      updateConfig,
      resetConfig,
      incrementAttempts,
    }),
    [
      assignment,
      assignmentFiles,
      attempts,
      bestMark,
      assignmentStats,
      memoOutput,
      markAllocator,
      readiness,
      policy,
      config,
      loading,
      refreshAssignmentStats,
      refreshAssignment,
      updateConfig,
      resetConfig,
      incrementAttempts,
    ],
  );

  if (loading || !assignment) return <AssignmentLoadingPlaceholder />;

  return (
    <AssignmentProvider value={value}>
      <Outlet />
    </AssignmentProvider>
  );
}
