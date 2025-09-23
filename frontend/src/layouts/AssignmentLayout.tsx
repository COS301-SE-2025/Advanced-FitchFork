// src/pages/modules/assignments/AssignmentLayout.tsx
import { useEffect, useState } from 'react';
import { useNavigate, useLocation, Outlet } from 'react-router-dom';
import { Dropdown, Button, Alert, Tag, Typography, Segmented, Modal, message } from 'antd';
import type { MenuProps } from 'antd';
import { DownloadOutlined } from '@ant-design/icons';

import { useModule } from '@/context/ModuleContext';
import { useAuth } from '@/context/AuthContext';
import { useAssignment } from '@/context/AssignmentContext';

import {
  closeAssignment,
  downloadAssignmentFile,
  openAssignment,
} from '@/services/modules/assignments';
import { generateMemoOutput } from '@/services/modules/assignments/memo-output';
import { generateMarkAllocator } from '@/services/modules/assignments/mark-allocator';
import { submitAssignment } from '@/services/modules/assignments/submissions/post';

import AssignmentStatusTag from '@/components/assignments/AssignmentStatusTag';
import { useUI } from '@/context/UIContext';
import SubmitAssignmentModal from '@/components/submissions/SubmitAssignmentModal';
import SetupChecklist from '@/components/assignments/SetupChecklist';
import Tip from '@/components/common/Tip';
import { SubmissionProgressOverlay } from '@/components/submissions';
import AssignmentSetup from '@/pages/modules/assignments/steps/AssignmentSetup';

const { Title, Paragraph } = Typography;

const AssignmentLayout = () => {
  const module = useModule();
  const { assignment, assignmentFiles, bestMark, attempts, readiness, policy, refreshAssignment } =
    useAssignment();
  const auth = useAuth();
  const { isMobile, isXxl } = useUI();
  const navigate = useNavigate();
  const location = useLocation();

  const [modalOpen, setModalOpen] = useState(false);
  const [checklistModalOpen, setChecklistModalOpen] = useState(false);
  const [wizardOpen, setWizardOpen] = useState(false);
  const [loading, setLoading] = useState(false);
  const [downloadingSpec, setDownloadingSpec] = useState(false);
  const [outletNonce, setOutletNonce] = useState(0);
  const [activeSubmissionId, setActiveSubmissionId] = useState<number | null>(null);
  const [deferredSubmit, setDeferredSubmit] = useState<null | (() => Promise<number | null>)>(null);

  // Overlay open state (overlay only shows progress; it does not submit)
  const [progressOpen, setProgressOpen] = useState(false);

  const basePath = `/modules/${module.id}/assignments/${assignment.id}`;
  const isStudentOrTutor = auth.isStudent(module.id) || auth.isTutor(module.id);
  const isTeachingStaff =
    auth.isLecturer(module.id) || auth.isAssistantLecturer(module.id) || auth.isAdmin;
  const isSetupStage = assignment.status === 'setup';
  const isAssignmentReady = readiness?.is_ready ?? false;

  const isOnSubmissions =
    location.pathname.startsWith(`${basePath}/submissions`) || location.pathname === `${basePath}`;
  const isUnlimitedAttempts = !!policy && !policy.limit_attempts;
  const attemptsExhausted =
    !!policy?.limit_attempts && !!attempts && (attempts.remaining ?? 0) <= 0;

  const showHeaderCard = !isMobile || (isMobile && isOnSubmissions);
  const canShowSetupPanel = isTeachingStaff && !isAssignmentReady && isXxl;
  const showSetupQuickAccess = isTeachingStaff && !isAssignmentReady;

  const segmentsConfig = [
    {
      value: `${basePath}/submissions`,
      label: 'Submissions',
      show: true,
      disabled: !isAssignmentReady,
    },
    { value: `${basePath}/tickets`, label: 'Tickets', show: true, disabled: !isAssignmentReady },
    ...(isTeachingStaff
      ? [
          { value: `${basePath}/tasks`, label: 'Tasks', show: true, disabled: false },
          {
            value: `${basePath}/grades`,
            label: 'Grades',
            show: true,
            disabled: !isAssignmentReady,
          },
          {
            value: `${basePath}/plagiarism`,
            label: 'Plagiarism',
            show: true,
            disabled: !isAssignmentReady,
          },
          {
            value: `${basePath}/statistics`,
            label: 'Statistics',
            show: true,
            disabled: !isAssignmentReady,
          },
          { value: `${basePath}/config`, label: 'Files & Config', show: true, disabled: false },
        ]
      : []),
  ];

  const segments = segmentsConfig.filter((seg) => seg.show).map(({ show, ...seg }) => seg);
  const isBasePath = location.pathname === basePath || location.pathname === `${basePath}/`;
  const normalizedPathname = isBasePath ? `${basePath}/submissions` : location.pathname;
  const shouldShowTabs = !isMobile && segments.length > 0;

  const candidateActiveKey =
    segments.find(
      (seg) => normalizedPathname === seg.value || normalizedPathname.startsWith(seg.value + '/'),
    )?.value ||
    segments[0]?.value ||
    normalizedPathname;

  const firstEnabledSegment = segments.find((seg) => !seg.disabled);
  const activeKey = segments.find((seg) => seg.value === candidateActiveKey && seg.disabled)
    ? (firstEnabledSegment?.value ?? candidateActiveKey)
    : candidateActiveKey;

  const segmentsClickable = segments.map((seg) => ({
    ...seg,
    label: (
      <span
        onClick={() => {
          if (seg.disabled) return;
          navigate(seg.value);
          if (activeKey === seg.value) {
            setOutletNonce((n) => n + 1);
            refreshAssignment?.();
          }
        }}
        style={{ display: 'inline-block', width: '100%' }}
      >
        {seg.label}
      </span>
    ),
  }));

  const handleOpenAssignment = async () => {
    setLoading(true);
    const hide = message.loading('Opening assignment...');
    try {
      const res = await openAssignment(module.id, assignment.id);
      hide();
      if (res.success) {
        await refreshAssignment();
        message.success('Assignment opened successfully');
      } else {
        message.error(res.message || 'Failed to open assignment');
      }
    } catch {
      hide();
      message.error('Unexpected error while opening assignment');
    } finally {
      setLoading(false);
    }
  };

  const handleCloseAssignment = async () => {
    setLoading(true);
    const hide = message.loading('Closing assignment...');
    try {
      const res = await closeAssignment(module.id, assignment.id);
      hide();
      if (res.success) {
        await refreshAssignment();
        message.success('Assignment closed successfully');
      } else {
        message.error(res.message || 'Failed to close assignment');
      }
    } catch {
      hide();
      message.error('Unexpected error while closing assignment');
    } finally {
      setLoading(false);
    }
  };

  const handleGenerateMemoOutput = async () => {
    setLoading(true);
    const hide = message.loading('Generating memo ouptut...');
    try {
      const res = await generateMemoOutput(module.id, assignment.id);
      hide();
      if (res.success) {
        await refreshAssignment();
        message.success('Memo output generated');
      } else {
        message.error(res.message || 'Failed to generate memo output');
      }
    } catch {
      hide();
      message.error('An unexpected error occurred');
    } finally {
      setLoading(false);
    }
  };

  const handleGenerateMarkAllocator = async () => {
    setLoading(true);
    const hide = message.loading('Generating mark allocator...');
    try {
      const res = await generateMarkAllocator(module.id, assignment.id);
      hide();
      if (res.success) {
        await refreshAssignment();
        message.success('Mark allocator generated');
      } else {
        message.error(res.message || 'Failed to generate mark allocator');
      }
    } catch {
      hide();
      message.error('Failed to generate mark allocator');
    } finally {
      setLoading(false);
    }
  };

  // Submit immediately here; overlay only shows progress
  // Submit immediately here; overlay only shows progress
  const handleSubmitAssignment = async (
    file: File,
    isPractice: boolean,
    attestsOwnership: boolean,
  ) => {
    setModalOpen(false);
    setProgressOpen(true);

    // Build a deferred thunk the overlay can call when WS is ready (or fallback fires)
    setDeferredSubmit(() => async () => {
      try {
        const res = await submitAssignment(
          module.id,
          assignment.id,
          file,
          isPractice,
          attestsOwnership,
          true,
        );

        const newId = (res as any)?.data?.id as number | undefined;
        if (newId) {
          setActiveSubmissionId(newId);
        }

        if (!res.success) {
          message.error(res.message || 'Submission failed');
          // Close overlay because we won’t get WS updates if submission failed
          setProgressOpen(false);
          setActiveSubmissionId(null);
          return null;
        }

        return newId ?? null;
      } catch {
        message.error('Submission failed');
        setProgressOpen(false);
        setActiveSubmissionId(null);
        return null;
      }
    });
  };

  const handleDownloadSpec = async () => {
    const specFile = assignmentFiles?.find((f) => f.file_type === 'spec');
    if (!specFile) {
      message.error('No specification file found for this assignment.');
      return;
    }
    setDownloadingSpec(true);
    const hide = message.loading('Starting specification download...');
    try {
      await downloadAssignmentFile(module.id, assignment.id, specFile.id);
      message.success('Download started');
    } catch {
      message.error('Failed to download specification');
    } finally {
      hide();
      setDownloadingSpec(false);
    }
  };

  type PrimaryActionKey = 'memo' | 'mark' | 'submit';
  type PrimaryAction = {
    key: PrimaryActionKey;
    label: string;
    onClick: () => void | Promise<void>;
  };

  const hasRequiredFilesForMemoOutput = Boolean(
    readiness?.memo_present &&
      readiness?.makefile_present &&
      (readiness?.submission_mode === 'gatlam'
        ? readiness?.interpreter_present
        : readiness?.main_present),
  );
  const hasAtLeastOneTask = Boolean(readiness?.tasks_present);

  const shouldOfferMemoAction = Boolean(
    isTeachingStaff &&
      policy?.submission_mode === 'manual' &&
      hasRequiredFilesForMemoOutput &&
      hasAtLeastOneTask,
  );
  const shouldOfferMarkAction = Boolean(
    isTeachingStaff && policy?.submission_mode === 'manual' && readiness?.memo_output_present,
  );

  const canGenerateMemoOutput = shouldOfferMemoAction && !readiness?.memo_output_present;
  const canGenerateMarkAllocator = shouldOfferMarkAction && !readiness?.mark_allocator_present;
  const canSubmitAssignment = assignment.status !== 'setup';

  const primaryAction: PrimaryAction | null = canGenerateMemoOutput
    ? { key: 'memo', label: 'Generate Memo Output', onClick: handleGenerateMemoOutput }
    : canGenerateMarkAllocator
      ? { key: 'mark', label: 'Generate Mark Allocator', onClick: handleGenerateMarkAllocator }
      : canSubmitAssignment && (isStudentOrTutor || isTeachingStaff)
        ? { key: 'submit', label: 'Submit Assignment', onClick: () => setModalOpen(true) }
        : null;

  const submitSection: MenuProps['items'] = [];
  if (isTeachingStaff && canSubmitAssignment && primaryAction?.key !== 'submit') {
    submitSection.push({
      key: 'submit',
      label: 'Submit Assignment',
      onClick: () => setModalOpen(true),
      disabled: loading,
    });
  }

  const manualActionsSection: MenuProps['items'] = [];
  if (shouldOfferMemoAction && primaryAction?.key !== 'memo') {
    manualActionsSection.push({
      key: 'memo',
      label: 'Generate Memo Output',
      onClick: handleGenerateMemoOutput,
      disabled: loading,
    });
  }
  if (shouldOfferMarkAction && primaryAction?.key !== 'mark') {
    manualActionsSection.push({
      key: 'mark',
      label: 'Generate Mark Allocator',
      onClick: handleGenerateMarkAllocator,
      disabled: loading,
    });
  }

  const managementSection: MenuProps['items'] = [
    {
      key: 'open',
      label: 'Open Assignment',
      onClick: handleOpenAssignment,
      disabled: loading || !['ready', 'closed', 'archived'].includes(assignment?.status ?? ''),
    },
    {
      key: 'close',
      label: 'Close Assignment',
      onClick: handleCloseAssignment,
      disabled: loading || assignment?.status !== 'open',
    },
  ];

  const archiveSection: MenuProps['items'] = [
    { key: 'archive', label: 'Archive Assignment', disabled: loading },
    { key: 'delete', label: 'Delete Assignment', danger: true, disabled: loading },
  ];

  const sectionsForMenu = [
    submitSection,
    manualActionsSection,
    managementSection,
    archiveSection,
  ].filter((section) => section.length > 0);

  const menuItems: MenuProps['items'] = [];
  sectionsForMenu.forEach((section, index) => {
    menuItems.push(...section);
    if (index < sectionsForMenu.length - 1) menuItems.push({ type: 'divider' });
  });

  const isSetupIncomplete = isSetupStage && !isAssignmentReady;

  const handleNavigateTo = (path: string) => navigate(path);
  const handleNavigateFromModal = (path: string) => {
    setChecklistModalOpen(false);
    navigate(path);
  };
  const handleLaunchWizard = (fromModal?: boolean) => {
    if (fromModal) setChecklistModalOpen(false);
    setWizardOpen(true);
  };
  const handleOpenConfig = (fromModal?: boolean) => {
    if (fromModal) setChecklistModalOpen(false);
    navigate(`${basePath}/config`);
  };

  useEffect(() => {
    if (!isAssignmentReady) return;
    if (checklistModalOpen) setChecklistModalOpen(false);
    if (wizardOpen) setWizardOpen(false);
  }, [isAssignmentReady, checklistModalOpen, wizardOpen]);

  useEffect(() => {
    if (isAssignmentReady) return;
    const restrictedPrefixes = [
      `${basePath}/submissions`,
      `${basePath}/tickets`,
      `${basePath}/grades`,
      `${basePath}/plagiarism`,
    ];
    const isRestricted = restrictedPrefixes.some(
      (prefix) => location.pathname === prefix || location.pathname.startsWith(`${prefix}/`),
    );
    if (!isRestricted) return;
    if (isTeachingStaff) {
      navigate(`${basePath}/tasks`, { replace: true });
    } else {
      navigate(`/modules/${module.id}/assignments`, { replace: true });
    }
  }, [isAssignmentReady, location.pathname, basePath, isTeachingStaff, module.id, navigate]);

  // Submit button disabled state (no “lock” logic now)
  const submitDisabled = loading || attemptsExhausted || !canSubmitAssignment;

  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="flex-1 overflow-y-auto px-4 pt-4 mb-4 h-full">
        <div className="flex h-full flex-col gap-4 lg:flex-row lg:items-start lg:gap-6">
          <div className="flex-1 flex flex-col gap-4 h-full">
            {showHeaderCard && (
              <div className="bg-white dark:bg-gray-900 border-gray-200 dark:border-gray-800 mb-0 p-4 rounded-md border ">
                <div className="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-6">
                  {/* Left section: Info */}
                  <div className="flex-1 space-y-4">
                    <div className="flex flex-wrap items-center gap-3">
                      <Title
                        level={3}
                        className="!mb-0 !text-gray-900 dark:!text-gray-100 !leading-none !text-2xl"
                      >
                        {assignment.name}
                      </Title>

                      <div className="flex items-center">
                        <AssignmentStatusTag status={assignment.status} />
                      </div>

                      {auth.isStudent(module.id) && bestMark && (
                        <Tag
                          color="green"
                          className="!text-xs !font-medium !h-6 !px-2 !flex items-center"
                        >
                          Best Mark: {Math.round((bestMark.earned / bestMark.total) * 100)}%
                        </Tag>
                      )}

                      {/* ---- Attempts ---- */}
                      {auth.isStudent(module.id) &&
                        (isUnlimitedAttempts ? (
                          <Tag
                            color="blue"
                            className="!text-xs !font-medium !h-6 !px-2 !flex items-center"
                            title="Unlimited attempts"
                          >
                            Unlimited Attempts
                          </Tag>
                        ) : attempts ? (
                          <Tag
                            color={attemptsExhausted ? 'red' : 'blue'}
                            className="!text-xs !font-medium !h-6 !px-2 !flex items-center"
                            title={
                              attemptsExhausted
                                ? 'No attempts remaining'
                                : `${attempts.remaining ?? 0} attempt(s) remaining`
                            }
                          >
                            Attempts: {attempts.used}/{attempts.max}
                          </Tag>
                        ) : null)}
                    </div>

                    {assignment.description?.length > 0 && (
                      <Paragraph className="!text-sm !text-gray-600 dark:!text-gray-400">
                        {assignment.description}
                      </Paragraph>
                    )}

                    {assignmentFiles?.some((f) => f.file_type === 'spec') && (
                      <Button
                        type="link"
                        onClick={handleDownloadSpec}
                        icon={<DownloadOutlined />}
                        size="small"
                        className="!p-0"
                        loading={downloadingSpec}
                        disabled={downloadingSpec}
                      >
                        Download Specification
                      </Button>
                    )}
                  </div>

                  {/* Right section: Actions */}
                  <div className="flex flex-col items-start sm:items-end gap-2 w-full sm:w-auto">
                    {primaryAction &&
                      (isTeachingStaff ? (
                        <Dropdown.Button
                          menu={{ items: menuItems }}
                          type="primary"
                          disabled={loading}
                          onClick={() => {
                            if (loading) return;
                            void primaryAction.onClick();
                          }}
                          loading={loading}
                          className="w-full sm:w-auto"
                          rootClassName="!w-full [&>button:first-child]:w-full"
                        >
                          {primaryAction.label}
                        </Dropdown.Button>
                      ) : (
                        <Button
                          type="primary"
                          onClick={() => {
                            if (submitDisabled || !primaryAction) return;
                            if (primaryAction.key === 'submit') {
                              setModalOpen(true);
                            } else {
                              void primaryAction.onClick();
                            }
                          }}
                          loading={loading}
                          className="w-full sm:w-auto"
                          disabled={submitDisabled}
                        >
                          {primaryAction.label}
                        </Button>
                      ))}

                    {showSetupQuickAccess && !canShowSetupPanel && (
                      <Button
                        type="default"
                        className="w-full sm:w-auto"
                        onClick={() => setChecklistModalOpen(true)}
                      >
                        View setup requirements
                      </Button>
                    )}
                  </div>
                </div>
                {shouldShowTabs && (
                  <div className="hidden md:block mt-4">
                    <Segmented
                      options={segmentsClickable}
                      value={activeKey}
                      onChange={(key) => navigate(key as string)}
                      size="middle"
                      block
                      className="dark:!bg-gray-950"
                    />
                  </div>
                )}
              </div>
            )}

            {auth.isStudent(module.id) &&
              assignment.due_date &&
              (() => {
                const now = new Date();
                const due = new Date(assignment.due_date);

                if (now <= due) return null; // still before due

                const late = policy?.late;

                if (!late || !late.allow_late_submissions) {
                  return (
                    <Alert
                      message="Past Due Date"
                      description="Submissions after the due date are not accepted."
                      type="error"
                      showIcon
                    />
                  );
                }

                const graceEnd = new Date(due.getTime() + late.late_window_minutes * 60 * 1000);

                if (now > graceEnd) {
                  return (
                    <Alert
                      message="Late Window Expired"
                      description="The late submission window has ended. No further submissions are accepted."
                      type="error"
                      showIcon
                    />
                  );
                }

                return (
                  <Alert
                    message="Late Submission Window Active"
                    description={`Submissions are still accepted until ${graceEnd.toLocaleString()}, 
          but marks are capped at ${late.late_max_percent}% of the total.`}
                    type="warning"
                    showIcon
                  />
                );
              })()}

            <div key={`${activeKey}:${outletNonce}`} className="h-full">
              <Outlet />
            </div>
          </div>

          {canShowSetupPanel && (
            <aside className="w-full lg:max-w-xs xl:max-w-sm lg:sticky lg:top-0 lg:self-start h-full">
              <div className="flex h-full flex-col gap-4 bg-white dark:bg-gray-900 border-2 border-dashed border-gray-200 dark:border-gray-800 rounded-xl p-4 sm:p-5">
                <div className="space-y-1">
                  <div className="flex items-center justify-start gap-2">
                    <Title level={4} className="!m-0 !text-gray-900 dark:!text-gray-100">
                      Setup requirements
                    </Title>
                    <Tip iconOnly newTab to="/help/assignments/setup" text="Open setup guide" />
                  </div>
                  <Paragraph className="!m-0 !text-sm !text-gray-600 dark:!text-gray-400">
                    {isSetupIncomplete
                      ? 'Finish the items below to make the assignment ready.'
                      : 'Setup items stay visible so you can spot regressions quickly.'}
                  </Paragraph>
                </div>

                <SetupChecklist
                  readiness={readiness ?? null}
                  basePath={basePath}
                  loading={loading}
                  shouldOfferMemoAction={shouldOfferMemoAction}
                  shouldOfferMarkAction={shouldOfferMarkAction}
                  onGenerateMemo={handleGenerateMemoOutput}
                  onGenerateMark={handleGenerateMarkAllocator}
                  onNavigate={handleNavigateTo}
                />

                <div className="flex flex-col gap-2 sm:flex-row lg:flex-col lg:mt-auto">
                  <Button
                    type="primary"
                    size="middle"
                    className="!w-full"
                    onClick={() => handleLaunchWizard()}
                    loading={loading}
                  >
                    Launch setup wizard
                  </Button>
                  <Button
                    type="default"
                    size="middle"
                    className="!w-full"
                    onClick={() => handleOpenConfig()}
                  >
                    Review configuration
                  </Button>
                </div>
              </div>
            </aside>
          )}
        </div>

        {/* Modal to pick the file */}
        <SubmitAssignmentModal
          open={modalOpen}
          onClose={() => setModalOpen(false)}
          onSubmit={handleSubmitAssignment}
          loading={loading}
          title={`Submit: ${assignment.name}`}
          accept=".zip,.tar,.gz,.tgz"
          maxSizeMB={50}
          defaultIsPractice={false}
          allowPractice={policy?.allow_practice_submissions && !auth.isStaff(module.id)}
        />

        <AssignmentSetup
          open={wizardOpen}
          onClose={() => setWizardOpen(false)}
          assignmentId={assignment.id}
          module={module}
          onDone={async () => {
            await refreshAssignment();
            setWizardOpen(false);
          }}
        />

        {progressOpen && auth.user?.id && (
          <SubmissionProgressOverlay
            moduleId={module.id}
            assignmentId={assignment.id}
            userId={auth.user.id}
            submissionId={activeSubmissionId ?? undefined}
            triggerSubmit={deferredSubmit ?? undefined}
            wsConnectTimeoutMs={2500} // fallback if WS doesn’t connect quickly
            onClose={() => {
              setProgressOpen(false);
              setActiveSubmissionId(null);
              setDeferredSubmit(null);
              refreshAssignment?.();
            }}
            onDone={(submissionId) => {
              setProgressOpen(false);
              setActiveSubmissionId(null);
              setDeferredSubmit(null);
              refreshAssignment?.();
              navigate(
                `/modules/${module.id}/assignments/${assignment.id}/submissions/${submissionId}`,
                { replace: true },
              );
            }}
          />
        )}

        <Modal
          title={
            <div className="flex items-center justify-start gap-2">
              <Title level={4} className="!m-0 !text-gray-900 dark:!text-gray-100">
                Setup requirements
              </Title>
              <Tip iconOnly newTab to="/help/assignments/setup" text="Open setup guide" />
            </div>
          }
          open={showSetupQuickAccess && !canShowSetupPanel && checklistModalOpen}
          onCancel={() => setChecklistModalOpen(false)}
          footer={null}
          destroyOnHidden
          width={640}
        >
          <div className="flex flex-col gap-4">
            <Paragraph className="!m-0 !text-sm !text-gray-600 dark:!text-gray-400">
              {isSetupIncomplete
                ? 'Finish the items below to make the assignment ready.'
                : 'Setup items stay visible so you can spot regressions quickly.'}
            </Paragraph>

            <SetupChecklist
              readiness={readiness ?? null}
              basePath={basePath}
              loading={loading}
              shouldOfferMemoAction={shouldOfferMemoAction}
              shouldOfferMarkAction={shouldOfferMarkAction}
              onGenerateMemo={handleGenerateMemoOutput}
              onGenerateMark={handleGenerateMarkAllocator}
              onNavigate={handleNavigateFromModal}
            />

            <div className="flex flex-col gap-2 sm:flex-row">
              <Button
                type="primary"
                className="!w-full"
                size="middle"
                onClick={() => handleLaunchWizard(true)}
                loading={loading}
              >
                Launch setup wizard
              </Button>
              <Button
                type="default"
                className="!w-full"
                size="middle"
                onClick={() => handleOpenConfig(true)}
              >
                Review configuration
              </Button>
            </div>
          </div>
        </Modal>
      </div>
    </div>
  );
};

export default AssignmentLayout;
