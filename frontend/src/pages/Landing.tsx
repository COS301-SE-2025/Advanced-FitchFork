import type { ReactNode } from 'react';
import { Button, Typography, Row, Col, Card, Carousel, Space, Tag, Steps } from 'antd';
import {
  CheckCircleOutlined,
  BulbFilled,
  BulbOutlined,
  RocketOutlined,
  SafetyCertificateOutlined,
  LineChartOutlined,
  ThunderboltOutlined,
  CodeOutlined,
  RobotOutlined,
  DashboardOutlined,
  CloudUploadOutlined,
  ScheduleOutlined,
  ExperimentOutlined,
  MessageOutlined,
} from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useTheme } from '@/context/ThemeContext';
import TiltScreenshot from '@/components/TiltScreenshot';

const { Title, Paragraph, Text } = Typography;

const lightImages = [
  '/screenshots/light/student_dashboard.png',
  '/screenshots/light/lecturer_dashboard.png',
  '/screenshots/light/submissions.png',
  '/screenshots/light/tasks.png',
  '/screenshots/light/config.png',
  '/screenshots/light/moss.png',
  '/screenshots/light/submit_result.png',
  '/screenshots/light/tickets.png',
  '/screenshots/light/session.png',
  '/screenshots/light/diffs.png',
  '/screenshots/light/graph_control.png',
  '/screenshots/light/staff.png',
];

const darkImages = [
  '/screenshots/dark/student_dashboard.png',
  '/screenshots/dark/lecturer_dashboard.png',
  '/screenshots/dark/submissions.png',
  '/screenshots/dark/tasks.png',
  '/screenshots/dark/config.png',
  '/screenshots/dark/moss.png',
  '/screenshots/dark/submit_result.png',
  '/screenshots/dark/tickets.png',
  '/screenshots/dark/session.png',
  '/screenshots/dark/diffs.png',
  '/screenshots/dark/graph_control.png',
  '/screenshots/dark/staff.png',
];

const sectionImages: Record<string, { light: string; dark: string }> = {
  containers: {
    light: '/screenshots/light/execution_controls.png',
    dark: '/screenshots/dark/execution_controls.png',
  },
  plagiarism: {
    light: '/screenshots/light/graph.png',
    dark: '/screenshots/dark/graph.png',
  },
  analytics: {
    light: '/screenshots/light/stats.png',
    dark: '/screenshots/dark/stats.png',
  },
};

const Landing = () => {
  const navigate = useNavigate();
  const { isDarkMode, setMode } = useTheme();

  const toggleTheme = () => setMode(isDarkMode ? 'light' : 'dark');
  const carouselImages = isDarkMode ? darkImages : lightImages;

  type ImagePlaceholderProps = { label: string };

  const ImagePlaceholder = ({ label }: ImagePlaceholderProps) => (
    <div className="relative w-full aspect-[16/10] rounded-2xl border border-dashed border-gray-300 dark:border-gray-700 bg-gradient-to-br from-blue-100/70 via-white to-purple-100/60 dark:from-gray-900/80 dark:via-gray-950 dark:to-gray-900/70 flex items-center justify-center">
      <span className="text-sm font-medium text-gray-500 dark:text-gray-400">
        Screenshot placeholder — {label}
      </span>
    </div>
  );

  type HighlightStat = {
    icon: ReactNode;
    label: string;
    description: string;
    accent: 'blue' | 'purple' | 'emerald';
  };

  const stats: HighlightStat[] = [
    {
      icon: <ThunderboltOutlined />,
      label: '12k submissions / hour',
      description: 'Container pools auto-scale for deadline spikes.',
      accent: 'blue',
    },
    {
      icon: <RocketOutlined />,
      label: 'Launch in < 5 minutes',
      description: 'Guided setup checks configs before students arrive.',
      accent: 'purple',
    },
    {
      icon: <LineChartOutlined />,
      label: '9 analytics surfaces',
      description: 'Stats, memo outputs, and exports keep moderation fast.',
      accent: 'emerald',
    },
  ];

  const statAccentStyles: Record<HighlightStat['accent'], { text: string; bg: string }> = {
    blue: { text: 'text-blue-500', bg: 'bg-blue-500/10 dark:bg-blue-500/20' },
    purple: { text: 'text-purple-500', bg: 'bg-purple-500/10 dark:bg-purple-500/20' },
    emerald: { text: 'text-emerald-500', bg: 'bg-emerald-500/10 dark:bg-emerald-500/20' },
  };

  const languages = [
    { name: 'Java', icon: '/languages/java.svg' },
    { name: 'C++', icon: '/languages/cpp.svg' },
    { name: 'C', icon: '/languages/c.svg' },
    { name: 'Go', icon: '/languages/go.svg' },
    { name: 'Rust', icon: '/languages/rust.svg' },
    { name: 'Python', icon: '/languages/python.svg' },
  ];

  const gatlamSteps = [
    {
      title: 'Seed the rubric',
      description: 'Set mark weights and sample outputs with the allocator.',
      icon: <CodeOutlined />,
    },
    {
      title: 'Stress test code',
      description:
        'GATLAM mutates the runner executing the code before weak submissions slip through.',
      icon: <ExperimentOutlined />,
    },
    {
      title: 'Deliver Gemini feedback',
      description: 'Gemini prompts are hardened for great hints without prompt injection.',
      icon: <MessageOutlined />,
    },
    {
      title: 'Human moderation',
      description: 'Lecturers review flagged edge cases with full traceability.',
      icon: <CheckCircleOutlined />,
    },
  ];

  const submissionFlow = [
    {
      title: 'Uploaded',
      description: 'Students push a zip or repo and see the attempt instantly.',
      icon: <CloudUploadOutlined />,
    },
    {
      title: 'Queued',
      description: 'The pool assigns an isolated container with module limits applied.',
      icon: <ScheduleOutlined />,
    },
    {
      title: 'Executing',
      description: 'Compilation, tests, and custom scripts stream their progress.',
      icon: <ThunderboltOutlined />,
    },
    {
      title: 'Evaluated',
      description: 'Marks, memo outputs, and AI commentary land on the attempt.',
      icon: <MessageOutlined />,
    },
    {
      title: 'Published',
      description: 'Grades sync to analytics and student dashboards straight away.',
      icon: <LineChartOutlined />,
    },
  ];

  const GatlamProcess = () => (
    <Card
      bordered={false}
      className="h-full bg-gradient-to-br from-purple-500/10 via-purple-500/5 to-blue-500/10 dark:from-purple-500/20 dark:via-purple-500/10 dark:to-blue-500/20 border border-purple-300/40 dark:border-purple-500/30 shadow-sm"
    >
      <Title level={4} className="!mt-0 !mb-3 !text-purple-700 dark:!text-purple-200">
        How GATLAM adapts your marking
      </Title>
      <Steps
        direction="vertical"
        size="small"
        items={gatlamSteps.map((step) => ({
          title: step.title,
          description: step.description,
          icon: step.icon,
        }))}
      />
      <Paragraph className="!mt-4 !mb-0 !text-gray-600 dark:!text-gray-300">
        Staff stay in control—AI assists, markers approve.
      </Paragraph>
    </Card>
  );

  const SubmissionLifecycle = () => (
    <Card
      bordered={false}
      className="h-full bg-gradient-to-br from-blue-500/10 via-blue-500/5 to-emerald-500/10 dark:from-blue-500/15 dark:via-blue-500/10 dark:to-emerald-500/20 border border-blue-300/40 dark:border-blue-500/30 shadow-sm"
    >
      <Title level={4} className="!mt-0 !mb-3 !text-blue-700 dark:!text-blue-200">
        Submission lifecycle at a glance
      </Title>
      <Steps
        direction="vertical"
        size="small"
        items={submissionFlow.map((step) => ({
          title: step.title,
          description: step.description,
          icon: step.icon,
        }))}
      />
      <Paragraph className="!mt-4 !mb-0 !text-gray-600 dark:!text-gray-300">
        Live dashboards mirror each state change, helping staff step in before queues spike.
      </Paragraph>
    </Card>
  );

  const LanguagesVisual = () => (
    <div className="h-full rounded-2xl border border-cyan-300/40 dark:border-cyan-400/30 bg-gradient-to-br from-cyan-500/10 via-cyan-500/5 to-blue-500/10 dark:from-cyan-500/20 dark:via-cyan-500/10 dark:to-blue-500/20 p-6 shadow-sm">
      <Title level={4} className="!mt-0 !text-cyan-700 dark:!text-cyan-200">
        Language sandboxes ready to swap in
      </Title>
      <Paragraph className="!text-gray-600 dark:!text-gray-300">
        Pair each task with the right toolchain—no more lab image drift.
      </Paragraph>
      <div className="mt-6 grid grid-cols-2 gap-4">
        {languages.map((lang) => (
          <Card
            key={lang.name}
            bordered={false}
            className="bg-white/80 dark:bg-gray-950/80 border border-gray-200/40 dark:border-gray-800/40 text-center"
          >
            <div className="flex flex-col items-center gap-3">
              <span className="flex h-16 w-16 items-center justify-center rounded-full border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-950">
                <img src={lang.icon} alt={lang.name} className="h-12 w-12 object-contain" />
              </span>
              <Text className="!font-medium !text-gray-700 dark:!text-gray-200">{lang.name}</Text>
            </div>
          </Card>
        ))}
      </div>
      <Paragraph className="!mt-6 !mb-0 !text-gray-600 dark:!text-gray-300">
        Mix languages per task or roll out a single brief across multiple streams.
      </Paragraph>
    </div>
  );

  type FeatureSection = {
    key: string;
    icon: ReactNode;
    eyebrow: string;
    title: string;
    description: string;
    bullets: string[];
    imageLabel: string;
    imageAlign: 'left' | 'right';
    visual?: ReactNode;
    layout?: 'split' | 'process';
  };

  // New: pill component (used beside the icon)
  const DockerReadyPill = () => (
    <div className="flex items-center gap-2 rounded-full border border-blue-500/40 dark:border-blue-400/40 bg-white/80 dark:bg-gray-950/80 px-3 py-1.5 shadow-sm">
      <span className="flex h-7 w-7 items-center justify-center rounded-full bg-blue-500/15">
        <img src="/icons/docker.svg" alt="Docker" className="h-4 w-4 object-contain" />
      </span>
      <Text className="!text-xs sm:!text-sm !font-medium !text-blue-600 dark:!text-blue-200">
        Docker runtime ready
      </Text>
    </div>
  );

  const featureSections: FeatureSection[] = [
    {
      key: 'containers',
      icon: <ThunderboltOutlined />,
      eyebrow: 'Containerized execution',
      title: 'Concurrent runs tuned for computer science cohorts',
      description:
        'Run every attempt inside hardened Docker pools that flex for your busiest lab weeks.',
      bullets: [
        'Auto-scale up to 12k runs an hour without extra ops work.',
        'Enforce module CPU, memory, and file policies on each container.',
        'Health pings warn staff before queues start to stack up.',
      ],
      imageLabel: 'Execution controls overview',
      imageAlign: 'right',
    },
    {
      key: 'automation',
      icon: <RobotOutlined />,
      eyebrow: 'Automated marking and AI feedback',
      title: 'Let GATLAM grade and explain every attempt',
      description: 'GATLAM stresses code and pairs it with Gemini feedback your markers can trust.',
      bullets: [
        'Seed the allocator once and let GATLAM evolve the rubric safely.',
        'Genetic trials try to break code before submissions slip through.',
        'Gemini responses are prompt-hardened for fast, student-ready hints.',
      ],
      imageLabel: 'GATLAM feedback sample',
      imageAlign: 'left',
      visual: <GatlamProcess />,
      layout: 'process',
    },
    {
      key: 'live-updates',
      icon: <DashboardOutlined />,
      eyebrow: 'Live submission visibility',
      title: 'Every status change, surfaced the moment it happens',
      description: 'Watch attempts move from upload to publish in real time and intervene early.',
      bullets: [
        'Dashboards reflect queued, executing, evaluated, and published states instantly.',
        'Ignore toggles, retries, and practice runs stay perfectly in sync.',
        'Alerts highlight queues that need a remark or manual nudge.',
      ],
      imageLabel: 'Submission activity feed',
      imageAlign: 'right',
      visual: <SubmissionLifecycle />,
      layout: 'process',
    },
    {
      key: 'languages',
      icon: <CodeOutlined />,
      eyebrow: 'Language coverage',
      title: 'Teach in the languages your department relies on',
      description: 'Ship a single brief to multiple streams while FitchFork handles toolchains.',
      bullets: [
        'Java, C++, C, Go, Rust, and Python ready out of the box.',
        'Swap in your own toolchain images when a module needs them.',
        'Target tasks per language without duplicating the assignment.',
      ],
      imageLabel: 'Runtime selector',
      imageAlign: 'left',
      visual: <LanguagesVisual />,
    },
    {
      key: 'plagiarism',
      icon: <SafetyCertificateOutlined />,
      eyebrow: 'Academic integrity',
      title: 'Stanford-grade plagiarism reports without leaving FitchFork',
      description:
        'Launch the Stanford similarity service from FitchFork and review interactive graphs in-app.',
      bullets: [
        'Submit batches to the Stanford service in a couple of clicks.',
        'Visualise matches, clusters, and risk scores without leaving the UI.',
        'Capture investigation notes alongside each flagged case.',
      ],
      imageLabel: 'Plagiarism graph explorer',
      imageAlign: 'right',
    },
    {
      key: 'analytics',
      icon: <LineChartOutlined />,
      eyebrow: 'Insightful statistics',
      title: 'Answer every review board question with one dashboard',
      description:
        'Pass rates, late trends, and task-level marks stay ready for every moderation meeting.',
      bullets: [
        'Compare medians, p75, and full-mark counts at a glance.',
        'Drill into task heatmaps to see where students struggle.',
        'Export gradebooks and memo output without extra scripts.',
      ],
      imageLabel: 'Assignment statistics summary',
      imageAlign: 'left',
    },
  ];

  const featureAccents = ['blue', 'purple', 'cyan', 'amber', 'rose', 'emerald'] as const;
  type FeatureAccent = (typeof featureAccents)[number];
  const featureAccentStyles: Record<FeatureAccent, { bg: string; text: string }> = {
    blue: { bg: 'bg-blue-500/10 dark:bg-blue-500/15', text: 'text-blue-600 dark:text-blue-300' },
    purple: {
      bg: 'bg-purple-500/10 dark:bg-purple-500/15',
      text: 'text-purple-600 dark:text-purple-300',
    },
    cyan: { bg: 'bg-cyan-500/10 dark:bg-cyan-500/15', text: 'text-cyan-600 dark:text-cyan-300' },
    amber: {
      bg: 'bg-amber-500/10 dark:bg-amber-500/15',
      text: 'text-amber-600 dark:text-amber-300',
    },
    rose: { bg: 'bg-rose-500/10 dark:bg-rose-500/15', text: 'text-rose-600 dark:text-rose-300' },
    emerald: {
      bg: 'bg-emerald-500/10 dark:bg-emerald-500/15',
      text: 'text-emerald-600 dark:text-emerald-300',
    },
  };

  return (
    <div
      className="h-screen overflow-y-auto overscroll-contain bg-white dark:bg-gray-950 text-gray-800 dark:text-gray-100"
      style={{ WebkitOverflowScrolling: 'touch' }}
    >
      <div className="min-h-full flex flex-col">
        {/* Header */}
        <header className="sticky top-0 z-50 bg-white/80 backdrop-blur dark:bg-gray-950/80 w-full">
          <div className="max-w-7xl mx-auto flex items-center justify-between py-6 px-6 min-w-0">
            <div className="flex items-center gap-2 sm:gap-3 min-w-0">
              <img
                src={isDarkMode ? '/ff_logo_dark.svg' : '/ff_logo_light.svg'}
                alt="FitchFork logo"
                className="h-8 w-8 shrink-0"
              />
              <Title
                level={3}
                className="!m-0 text-gray-900 dark:text-white whitespace-nowrap truncate max-w-[48vw] sm:max-w-none"
              >
                FitchFork
              </Title>
            </div>
            <div className="flex items-center gap-2 sm:gap-3 flex-nowrap shrink-0">
              <Button icon={isDarkMode ? <BulbFilled /> : <BulbOutlined />} onClick={toggleTheme} />
              <Button type="text" onClick={() => navigate('/login')}>
                Login
              </Button>
              <Button type="primary" onClick={() => navigate('/signup')}>
                Sign Up
              </Button>
            </div>
          </div>
        </header>

        {/* Main */}
        <main className="flex-1">
          {/* Hero */}
          <section className="px-6 py-16 bg-gray-50 dark:bg-gray-900">
            <div className="max-w-6xl mx-auto grid gap-10 lg:grid-cols-[minmax(0,1fr),minmax(0,1.05fr)] items-center">
              <div className="text-center lg:text-left">
                <Tag color="blue" className="!px-4 !py-1 !text-sm !mx-auto lg:!mx-0">
                  Purpose-built for computer science departments
                </Tag>
                <Title className="!text-4xl sm:!text-5xl !font-semibold !text-gray-900 dark:!text-white !mt-5">
                  Automated coding assignments without the bottlenecks
                </Title>
                <Paragraph className="!text-lg !max-w-2xl !mx-auto lg:!mx-0 !text-gray-600 dark:!text-gray-300">
                  Launch practicals, run them in containers, and ship AI feedback that your cohort
                  can trust—all in one workspace.
                </Paragraph>
                <div className="mt-8 flex flex-wrap justify-center lg:justify-start gap-3 sm:gap-4">
                  <Button size="large" type="primary" onClick={() => navigate('/signup')}>
                    Start now
                  </Button>
                  <Button size="large" onClick={() => navigate('/login')}>
                    Explore the demo
                  </Button>
                </div>
              </div>
            </div>
          </section>

          <section className="px-6 py-12 bg-white dark:bg-gray-950">
            <div className="max-w-5xl mx-auto">
              <Row gutter={[24, 24]}>
                {stats.map((stat) => {
                  const accent = statAccentStyles[stat.accent];
                  return (
                    <Col key={stat.label} xs={24} md={8}>
                      <Card
                        bordered={false}
                        className="h-full bg-gray-50 dark:bg-gray-900 border border-gray-200/60 dark:border-gray-800/60 shadow-sm"
                      >
                        <Space align="start" size={16}>
                          <span
                            className={`inline-flex h-12 w-12 items-center justify-center rounded-full text-xl ${accent.bg} ${accent.text}`}
                          >
                            {stat.icon}
                          </span>
                          <div>
                            <Title level={4} className="!mb-1 !text-gray-900 dark:!text-white">
                              {stat.label}
                            </Title>
                            <Paragraph className="!mb-0 !text-gray-600 dark:!text-gray-300">
                              {stat.description}
                            </Paragraph>
                          </div>
                        </Space>
                      </Card>
                    </Col>
                  );
                })}
              </Row>
            </div>
          </section>

          {featureSections.map((section, index) => {
            const background =
              index % 2 === 0 ? 'bg-white dark:bg-gray-950' : 'bg-gray-50 dark:bg-gray-900';
            const accent = featureAccentStyles[featureAccents[index % featureAccents.length]];

            const defaultVisual = sectionImages[section.key] ? (
              <TiltScreenshot
                lightSrc={sectionImages[section.key].light}
                darkSrc={sectionImages[section.key].dark}
                alt={section.imageLabel}
                className="w-full"
              />
            ) : (
              <ImagePlaceholder label={section.imageLabel} />
            );

            // Updated: no overlay pill on the image
            const visualNode = section.visual ?? defaultVisual;

            if (section.layout === 'process') {
              return (
                <section key={section.key} className={`px-6 py-20 ${background}`}>
                  <div className="max-w-6xl mx-auto">
                    <div className="flex flex-col items-center text-center">
                      <span
                        className={`inline-flex h-14 w-14 items-center justify-center rounded-full text-2xl ${accent.bg} ${accent.text}`}
                      >
                        {section.icon}
                      </span>
                      <Tag
                        color="default"
                        className="!mt-6 !border-none !bg-gray-200/70 dark:!bg-gray-800/70 !text-gray-700 dark:!text-gray-300 !px-3 !py-1"
                      >
                        {section.eyebrow}
                      </Tag>
                      <Title level={2} className="!mt-4 !text-3xl !text-gray-900 dark:!text-white">
                        {section.title}
                      </Title>
                      <Paragraph className="!max-w-3xl !text-gray-600 dark:!text-gray-300">
                        {section.description}
                      </Paragraph>
                      <ul className="mt-6 space-y-3 w-full max-w-3xl text-gray-600 dark:text-gray-300 text-center list-none p-0">
                        {section.bullets.map((bullet) => (
                          <li key={bullet} className="flex items-center justify-center gap-3">
                            <CheckCircleOutlined className="text-blue-500 dark:text-blue-400" />
                            <span className="max-w-xl">{bullet}</span>
                          </li>
                        ))}
                      </ul>
                    </div>
                    <div className="mt-12 max-w-4xl mx-auto w-full">{visualNode}</div>
                  </div>
                </section>
              );
            }

            const textOrder = section.imageAlign === 'left' ? 2 : 1;
            const imageOrder = section.imageAlign === 'left' ? 1 : 2;

            return (
              <section key={section.key} className={`px-6 py-20 ${background}`}>
                <div className="max-w-6xl mx-auto">
                  <Row gutter={[32, 32]} align="middle">
                    <Col xs={24} lg={12} order={textOrder}>
                      {/* Updated: icon + pill row */}
                      <div className="flex items-center justify-between gap-3">
                        <span
                          className={`inline-flex h-12 w-12 items-center justify-center rounded-full text-xl ${accent.bg} ${accent.text}`}
                        >
                          {section.icon}
                        </span>
                        {section.key === 'containers' && <DockerReadyPill />}
                      </div>

                      <div className="mt-4">
                        <Tag
                          color="default"
                          className="!border-none !bg-gray-200/70 dark:!bg-gray-800/70 !text-gray-700 dark:!text-gray-300 !px-3 !py-1"
                        >
                          {section.eyebrow}
                        </Tag>
                      </div>
                      <Title level={2} className="!mt-4 !text-3xl !text-gray-900 dark:!text-white">
                        {section.title}
                      </Title>
                      <Paragraph className="!text-gray-600 dark:!text-gray-300">
                        {section.description}
                      </Paragraph>
                      <ul className="mt-6 space-y-3 text-left">
                        {section.bullets.map((bullet) => (
                          <li
                            key={bullet}
                            className="flex items-start gap-3 text-gray-600 dark:text-gray-300"
                          >
                            <CheckCircleOutlined className="mt-0.5 text-blue-500 dark:text-blue-400" />
                            <span>{bullet}</span>
                          </li>
                        ))}
                      </ul>
                    </Col>
                    <Col xs={24} lg={12} order={imageOrder}>
                      {visualNode}
                    </Col>
                  </Row>
                </div>
              </section>
            );
          })}

          <section className="px-6 py-20 bg-gradient-to-b from-white to-gray-100 dark:from-gray-950 dark:to-gray-900">
            <div className="max-w-5xl mx-auto text-center">
              <Title level={3} className="!text-gray-900 dark:!text-white">
                Product walkthrough snapshots
              </Title>
              <Paragraph className="!max-w-3xl !mx-auto !text-gray-600 dark:!text-gray-300">
                Capture the checklist, execution settings, live submissions, and analytics views to
                tell the full assignments story.
              </Paragraph>
            </div>
            <div className="mt-10 max-w-4xl mx-auto rounded-xl overflow-hidden border border-gray-200 dark:border-gray-800">
              <Carousel autoplay dotPosition="bottom">
                {carouselImages.map((src, index) => (
                  <div
                    key={index}
                    className="flex justify-center items-center bg-white dark:bg-gray-900"
                  >
                    <img
                      src={src}
                      alt={`Slide ${index + 1}`}
                      className="w-full h-auto aspect-[1590/942] object-contain"
                    />
                  </div>
                ))}
              </Carousel>
            </div>
          </section>

          <section className="px-6 py-20 bg-white dark:bg-gray-950">
            <div className="max-w-4xl mx-auto text-center">
              <Title level={3} className="!text-gray-900 dark:!text-white">
                Ready to run your next cohort with confidence?
              </Title>
              <Paragraph className="!text-gray-600 dark:!text-gray-300">
                Onboard lecturers, tutors, and students with guided setup and AI-powered grading
                that scales with your labs.
              </Paragraph>
              <div className="mt-6 flex flex-wrap justify-center gap-3">
                <Button size="large" type="primary" onClick={() => navigate('/signup')}>
                  Create my workspace
                </Button>
                <Button size="large" onClick={() => navigate('/login')}>
                  Book a walkthrough
                </Button>
              </div>
            </div>
          </section>
        </main>

        {/* Footer */}
        <footer className="text-center py-8 text-sm text-gray-400 dark:text-gray-600">
          © {new Date().getFullYear()} FitchFork. Built for education.
        </footer>
      </div>
    </div>
  );
};

export default Landing;
