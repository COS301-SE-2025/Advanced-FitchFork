export type TeamMember = {
  id: 'jacques' | 'luke' | 'richard' | 'aidan' | 'reece';
  name: string;
  role: string;
  summary: string;
  bio: string[];
  focusAreas: string[];
  highlights: string[];
  linkedin: string;
  github: string;
  location: string;
  avatar?: string;
};

export const teamMembers: TeamMember[] = [
  {
    id: 'jacques',
    name: 'Jacques Klooster',
    role: 'Team Lead & Frotned Dev',
    location: 'Cape Town, South Africa',
    summary:
      'Guides product vision, designed the ui/ux experience and lead the team via devops.',
    bio: [
      'Jacques leads our product direction, bringing a decade of experience supporting large programming cohorts.',
      'He partners closely with programme heads to ensure FitchFork removes friction from launch day through moderation.',
    ],
    focusAreas: ['Product strategy', 'Customer partnerships', 'Academic operations'],
    highlights: [
      'Scaled execution pools that now serve thousands of submissions per hour.',
      'Championed frictionless onboarding patterns for lecturers and teaching assistants.',
      'Advocates for human-in-the-loop oversight across every automated workflow.',
    ],
    linkedin: 'https://www.linkedin.com/in/jacquesklooster',
    github: 'https://github.com/jacqu3sk',
    avatar: '/team/jacques.webp',
  },
  {
    id: 'luke',
    name: 'Luke Gouws',
    role: 'AI Powered Evaluation & Integration',
    location: 'Johannesburg, South Africa',
    summary:
      'Owns the platform architecture with a focus on resilient infrastructure and developer velocity.',
    bio: [
      'Luke architects the container orchestration layer that powers FitchFork’s execution engine.',
      'He keeps the developer experience sharp so new modules and integrations land quickly.',
    ],
    focusAreas: ['Platform engineering', 'Container orchestration', 'Developer experience'],
    highlights: [
      'Designed FitchFork’s self-healing container pools with predictive autoscaling.',
      'Built the internal tooling that keeps deployments safe and observable.',
      'Guides our security posture across infrastructure and runtime sandboxes.',
    ],
    linkedin: 'https://www.linkedin.com/in/luke-gouws-4b07b7300/',
    github: 'https://github.com/CartographySilence',
    avatar: '/team/luke.webp',
  },
  {
    id: 'richard',
    name: 'Richard Kruse',
    role: 'Code Execution & System Security',
    location: 'Pretoria, South Africa',
    summary:
      'Translates lecturer feedback into product improvements and supports moderation pipelines.',
    bio: [
      'Richard helps programmes reimagine their assessment flow using FitchFork’s live analytics.',
      'He develops resources and workshops so staff can complete moderation with confidence.',
    ],
    focusAreas: ['Academic enablement', 'Moderation workflows', 'Change management'],
    highlights: [
      'Built the moderation playbooks used by FitchFork partner institutions.',
      'Led continuous training series for teaching assistants and academic staff.',
      'Connects university quality assurance teams with FitchFork roadmap decisions.',
    ],
    linkedin: 'https://www.linkedin.com/in/richard-kruse/',
    github: 'https://github.com/RKruse42',
    avatar: '/team/richard.webp',
  },
  {
    id: 'aidan',
    name: 'Aidan McKenzie',
    role: 'Backend Systems & API Development',
    location: 'Pretoria, South Africa',
    summary:
      'Crafts the instructor experience, aligning UI polish with accessibility and performance.',
    bio: [
      'Aidan ensures every lecturer workflow feels deliberate, from configuring tasks to reviewing analytics.',
      'He collaborates with customers to turn beta feedback into polished, production-ready experiences.',
    ],
    focusAreas: ['Product engineering', 'Design systems', 'Accessibility'],
    highlights: [
      'Rolled out the design language used across the FitchFork instructor console.',
      'Optimised key pages to respond instantly during peak submission periods.',
      'Introduced accessibility guardrails that keep our UI inclusive by default.',
    ],
    linkedin: 'https://www.linkedin.com/in/aidan-mckenzie-772730355/',
    github: 'https://github.com/RaiderRoss',
    avatar: '/team/aidan.webp',
  },
  {
    id: 'reece',
    name: 'Reece Jordaan',
    role: 'Automated Evaluation & API Infrastructure',
    location: 'Cape Town, South Africa',
    summary:
      'Leads our AI feedback and similarity detection research with an emphasis on transparency.',
    bio: [
      'Reece steers the evolution of GATLAM, balancing AI-assisted grading with human oversight.',
      'He partners with integrity offices to align FitchFork innovation with institutional policy.',
    ],
    focusAreas: ['Machine learning', 'Academic integrity', 'Ethical AI'],
    highlights: [
      'Delivered FitchFork’s explainable AI feedback pipeline for student attempts.',
      'Integrated plagiarism signals that link directly to instructor investigation tools.',
      'Chairs our internal review board for responsible AI deployment.',
    ],
    linkedin: 'https://www.linkedin.com/in/reecejordaan/',
    github: 'https://github.com/rxxim',
    avatar: '/team/reece.webp',
  },
];

export const heroSummary =
  'We are a multidisciplinary team building education-first infrastructure for automated programming assessments.';
