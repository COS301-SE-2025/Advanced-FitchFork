import { useMemo } from 'react';
import { Alert, Descriptions, List, Space, Typography, Tag } from 'antd';
import { useHelpToc } from '@/context/HelpContext';
import { useHelpPageHeader } from '@/pages/help/useHelpPageHeader';

const { Title, Paragraph, Text } = Typography;

const toc = [
  { key: 'overview', href: '#overview', title: 'What attendance sessions cover' },
  { key: 'create', href: '#create', title: 'Create a new session' },
  { key: 'run', href: '#run', title: 'Running a live session' },
  { key: 'student', href: '#student', title: 'Student check-in options' },
  { key: 'projector', href: '#projector', title: 'Projector mode & rotation' },
  { key: 'reporting', href: '#reporting', title: 'After the session' },
  { key: 'tips', href: '#tips', title: 'Tips & safeguards' },
];

export default function ModuleAttendanceHelp() {
  useHelpPageHeader('modules/attendance', { label: 'Attendance' });
  const ids = useMemo(() => toc.map((t) => t.href.slice(1)), []);

  useHelpToc({
    items: toc,
    ids,
    onMountScrollToHash: true,
  });

  return (
    <Space direction="vertical" size="large" className="w-full !p-0">
      <div>
        <Title level={2} className="mb-2">
          Attendance
        </Title>
        <Paragraph className="mb-0 text-gray-600 dark:text-gray-300">
          FitchFork attendance sessions give lecturers rotating join codes, projector queues, and
          guardrails like IP pinning. Students check in from the dashboard or the rotate-and-scan
          projector view, and staff can audit every attempt afterwards.
        </Paragraph>
      </div>

      <section id="overview" className="scroll-mt-32">
        <Title level={3}>What attendance sessions cover</Title>
        <Descriptions bordered size="middle" column={1} className="mt-3">
          <Descriptions.Item label="Where to find it">
            Inside a module choose <Text strong>Attendance</Text>. Staff see the session list; tutors
            and students access the most recent session.
          </Descriptions.Item>
          <Descriptions.Item label="Session anatomy">
            <ul className="list-disc pl-5 space-y-1">
              <li>
                <Text strong>Title</Text> – shows across the lecturer table, cards, and student
                prompts.
              </li>
              <li>
                <Text strong>Rotation (sec)</Text> – how often the join code and QR regenerate.
              </li>
              <li>
                <Text strong>Restrict by IP</Text> – limit check-ins to a lab subnet or to the
                creator&apos;s IP.
              </li>
              <li>
                <Text strong>Status</Text> – active sessions appear on the student dashboard and in
                projector mode.
              </li>
            </ul>
          </Descriptions.Item>
        </Descriptions>
      </section>

      <section id="create" className="scroll-mt-32">
        <Title level={3}>Create a new session</Title>
        <List
          size="small"
          bordered
          dataSource={[
            'Open the module → Attendance and select “New Session”.',
            'Set a clear title (e.g. Week 6 Practical) and whether the session should go live now.',
            'Choose a rotation window. 30–45 seconds gives students enough time without leaving stale codes.',
            'Toggle IP restrictions if you only want in-lab check-ins. For remote classes leave it off.',
            'Save to publish. Staff can edit the session later to pause, reword, or change restrictions.',
          ]}
          renderItem={(item) => <List.Item className="text-gray-700 dark:text-gray-200">{item}</List.Item>}
        />
      </section>

      <section id="run" className="scroll-mt-32">
        <Title level={3}>Running a live session</Title>
        <Paragraph>
          Use the staff table when you need bulk actions and history; switch to the grid for a quick
          glance. Each session card shows join stats, activity timeline, and shortcuts to projector
          mode or edit.
        </Paragraph>
        <Alert
          type="info"
          showIcon
          message="Closing a session"
          description="Toggle the Active switch to instantly hide the session from student dashboards without deleting your history."
        />
      </section>

      <section id="student" className="scroll-mt-32">
        <Title level={3}>Student check-in options</Title>
        <Paragraph>
          Students can open <Text code>/attendance/mark</Text> or tap the Attendance card on the
          dashboard. They enter the rotating alphanumeric code or scan the QR shown in projector
          mode. If IP restrictions are enabled FitchFork validates the request before recording the
          attempt.
        </Paragraph>
      </section>

      <section id="projector" className="scroll-mt-32">
        <Title level={3}>Projector mode &amp; rotation</Title>
        <Paragraph>
          Projector mode expands the join code and QR in a dark theme, ideal for lecture theatres. It
          refreshes automatically using the session’s rotation interval and highlights the time left
          before the next code swap. Use it to keep a queue visible while you monitor joins from the
          staff list.
        </Paragraph>
      </section>

      <section id="reporting" className="scroll-mt-32">
        <Title level={3}>After the session</Title>
        <Paragraph>
          Open a session to review the roster. You can export attendance, remove accidental check-ins,
          and see which device or IP submitted each attempt. Use the <Text strong>Projector</Text>
          button later if you need to reopen a session for late arrivals.
        </Paragraph>
      </section>

      <section id="tips" className="scroll-mt-32">
        <Title level={3}>Tips &amp; safeguards</Title>
        <List
          bordered
          size="small"
          dataSource={[
            'Create sessions ahead of time and keep them inactive until class starts.',
            'Use a shorter rotation (20–30s) with projector mode in large venues to limit code sharing.',
            'Enable IP restriction with a /24 subnet when you want on-campus only attendance.',
            'Use the creator IP pin to lock the session to the machine you opened in the lab.',
            'Archive unused sessions instead of deleting so you keep historic participation data.',
          ]}
          renderItem={(item, index) => (
            <List.Item className="text-gray-700 dark:text-gray-200">
              <Tag color="blue" className="mr-2">
                {index + 1}
              </Tag>
              <span>{item}</span>
            </List.Item>
          )}
        />
      </section>
    </Space>
  );
}
