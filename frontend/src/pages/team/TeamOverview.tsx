import { useRef, useState, type CSSProperties, type MouseEventHandler } from 'react';
import { GithubOutlined, LinkedinOutlined } from '@ant-design/icons';
import { Avatar, Button, Col, Row, Tag, Tooltip, Typography } from 'antd';

import type { TeamMember } from './teamData';
import { heroSummary, teamMembers } from './teamData';
import MarketingHeader from '@/components/marketing/MarketingHeader';

const { Title, Paragraph, Text } = Typography;

const TILT_EASE = 'cubic-bezier(0.22, 1, 0.36, 1)';

type TeamMemberCardProps = {
  member: TeamMember;
};

const TeamMemberCard = ({ member }: TeamMemberCardProps) => {
  const wrapperRef = useRef<HTMLDivElement | null>(null);
  const [tiltStyle, setTiltStyle] = useState<CSSProperties>({
    transform: 'translateZ(0) rotateX(0deg) rotateY(0deg) scale(1)',
    transition: `transform 240ms ${TILT_EASE}`,
  });

  const handleMove: MouseEventHandler<HTMLDivElement> = (event) => {
    const container = wrapperRef.current;
    if (!container) return;
    const rect = container.getBoundingClientRect();
    const px = (event.clientX - rect.left) / rect.width;
    const py = (event.clientY - rect.top) / rect.height;
    const rotX = (py - 0.5) * 12;
    const rotY = (0.5 - px) * 12;
    setTiltStyle({
      transform: `translateZ(-28px) rotateX(${rotX}deg) rotateY(${rotY}deg) scale(1.06)`,
      transition: 'transform 65ms linear',
    });
  };

  const handleLeave = () =>
    setTiltStyle({
      transform: 'translateZ(0) rotateX(0deg) rotateY(0deg) scale(1)',
      transition: `transform 260ms ${TILT_EASE}`,
    });

  return (
    <div
      ref={wrapperRef}
      style={{ perspective: '1300px' }}
      className="group h-full"
      onMouseMove={handleMove}
      onMouseLeave={handleLeave}
    >
      <div
        className="
          h-full transform-gpu rounded-3xl border-2 border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-800/30
          transition-[box-shadow] duration-200 ease-out
          hover:shadow-[0_0_65px_-25px_rgba(59,130,246,0.55)]
          dark:hover:shadow-[0_0_65px_-25px_rgba(59,130,246,0.4)]
          p-8 flex flex-col items-center gap-5 text-center
        "
        style={{ transformStyle: 'preserve-3d', ...tiltStyle }}
      >
        <Avatar
          size={160}
          src={member.avatar}
          className="bg-gradient-to-br from-blue-500 via-purple-500 to-emerald-500 text-white text-5xl font-semibold shadow-lg"
          alt={member.name}
        >
          {member.name.charAt(0)}
        </Avatar>
        <div>
          <Title level={4} className="!mb-1 !text-gray-900 dark:!text-white">
            {member.name}
          </Title>
          <Text className="!text-sm !font-medium !uppercase !tracking-wide !text-blue-600 dark:!text-blue-300">
            {member.role}
          </Text>
          <Paragraph className="!mt-2 !text-sm !text-gray-500 dark:!text-gray-400">
            {member.location}
          </Paragraph>
        </div>
        <div className="flex items-center justify-center gap-2">
          <Tooltip title="LinkedIn">
            <Button
              href={member.linkedin}
              target="_blank"
              rel="noopener noreferrer"
              shape="circle"
              icon={<LinkedinOutlined />}
            />
          </Tooltip>
          <Tooltip title="GitHub">
            <Button
              href={member.github}
              target="_blank"
              rel="noopener noreferrer"
              shape="circle"
              icon={<GithubOutlined />}
            />
          </Tooltip>
        </div>
      </div>
    </div>
  );
};

const TeamOverview = () => {
  return (
    <div
      className="h-screen overflow-y-auto overscroll-contain bg-white dark:bg-gray-950 text-gray-800 dark:text-gray-100"
      style={{ WebkitOverflowScrolling: 'touch' }}
    >
      <div className="min-h-full flex flex-col">
        <MarketingHeader />

        <main className="flex-1 bg-gray-50 dark:bg-gray-950">
          <div className="mx-auto flex max-w-6xl flex-col px-6 py-16">
            <header className="flex flex-col items-center text-center gap-5">
              <Tag color="blue" className="!px-3 !py-1 !text-xs tracking-wide uppercase">
                Meet the team
              </Tag>
              <Title className="!m-0 !text-4xl !font-semibold !text-gray-900 dark:!text-white">
                Builders behind FitchFork
              </Title>
              <Paragraph className="!m-0 !max-w-2xl !text-base !text-gray-600 dark:!text-gray-300">
                {heroSummary}
              </Paragraph>
            </header>

            <section className="mt-14">
              <Row gutter={[24, 24]} justify="center">
                {teamMembers.map((member) => (
                  <Col key={member.id} xs={24} md={12} xl={8}>
                    <TeamMemberCard member={member} />
                  </Col>
                ))}
              </Row>
            </section>
          </div>
        </main>

        <footer className="text-center py-8 text-sm text-gray-400 dark:text-gray-600">
          © {new Date().getFullYear()} FitchFork. Built for education.
        </footer>
      </div>
    </div>
  );
};

export default TeamOverview;
