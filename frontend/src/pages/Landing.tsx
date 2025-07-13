import { Button, Typography, Row, Col, Card, Carousel } from 'antd';
import {
  UserAddOutlined,
  UploadOutlined,
  CheckCircleOutlined,
  BulbFilled,
  BulbOutlined,
} from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useTheme } from '@/context/ThemeContext';

const { Title, Paragraph } = Typography;

// Light and dark mode image sets
const lightImages = [
  '/screenshots/light/1.webp',
  '/screenshots/light/2.webp',
  '/screenshots/light/3.webp',
  '/screenshots/light/4.webp',
  '/screenshots/light/5.webp',
  '/screenshots/light/6.webp',
  '/screenshots/light/7.webp',
];

const darkImages = [
  '/screenshots/dark/1.webp',
  '/screenshots/dark/2.webp',
  '/screenshots/dark/3.webp',
  '/screenshots/dark/4.webp',
  '/screenshots/dark/5.webp',
  '/screenshots/dark/6.webp',
  '/screenshots/dark/7.webp',
];

const Landing = () => {
  const navigate = useNavigate();
  const { isDarkMode, setMode } = useTheme();

  const toggleTheme = () => setMode(isDarkMode ? 'light' : 'dark');
  const carouselImages = isDarkMode ? darkImages : lightImages;

  return (
    <div className="!h-full !bg-white dark:!bg-gray-950 !text-gray-800 dark:!text-gray-100">
      <div className="!h-full !flex !flex-col">
        {/* Header */}
        <header className="sticky top-0 z-50 bg-white/80 backdrop-blur dark:bg-gray-950/80 w-full">
          <div className="max-w-7xl mx-auto flex items-center justify-between py-6 px-6">
            <div className="flex items-center gap-3">
              <img
                src={isDarkMode ? '/ff_logo_dark.svg' : '/ff_logo_light.svg'}
                alt="FitchFork logo"
                className="h-8 w-8"
              />

              <Title level={3} className="!m-0 text-gray-900 dark:text-white">
                FitchFork
              </Title>
            </div>
            <div className="flex items-center gap-3">
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

        {/* Main Content */}
        <main className="!flex-1 !overflow-y-auto">
          {/* Hero Section */}
          <section className="!text-center !px-6 !py-12 bg-gray-50 dark:bg-gray-900">
            <Title className="!text-4xl sm:!text-5xl !font-bold !text-gray-900 dark:!text-white !mb-4">
              Your Academic Workflow. Simplified.
            </Title>
            <Paragraph className="!text-lg !max-w-3xl !mx-auto !text-gray-600 dark:!text-gray-300">
              FitchFork is a unified platform that streamlines assignment creation, submission,
              grading, and feedback for students and educators. Fast, reliable, and purpose-built
              for modern academic environments.
            </Paragraph>
            <div className="!mt-6 !space-x-4">
              <Button size="large" type="primary" onClick={() => navigate('/signup')}>
                Get Started
              </Button>
              <Button size="large" onClick={() => navigate('/login')}>
                Already have an account?
              </Button>
            </div>
          </section>

          {/* Features Section */}
          <section className="!max-w-6xl !mx-auto !px-6 !py-16">
            <Title level={3} className="!text-center !mb-12">
              What You Can Do
            </Title>
            <Row gutter={[24, 24]} justify="center">
              <Col xs={24} sm={12} md={8}>
                <Card className="!bg-gray-100 dark:!bg-gray-800 !h-full !shadow hover:!shadow-lg !transition">
                  <UserAddOutlined style={{ fontSize: 36 }} />
                  <Title level={4} className="!mt-3">
                    Manage Roles & Modules
                  </Title>
                  <Paragraph>
                    Admins, lecturers, tutors, and students can manage their modules with flexible
                    permissions and visibility.
                  </Paragraph>
                </Card>
              </Col>
              <Col xs={24} sm={12} md={8}>
                <Card className="!bg-gray-100 dark:!bg-gray-800 !h-full !shadow hover:!shadow-lg !transition">
                  <UploadOutlined style={{ fontSize: 36 }} />
                  <Title level={4} className="!mt-3">
                    Submit & Track Assignments
                  </Title>
                  <Paragraph>
                    Students can upload their work and view submissions, feedback, and results—all
                    in one place.
                  </Paragraph>
                </Card>
              </Col>
              <Col xs={24} sm={12} md={8}>
                <Card className="!bg-gray-100 dark:!bg-gray-800 !h-full !shadow hover:!shadow-lg !transition">
                  <CheckCircleOutlined style={{ fontSize: 36 }} />
                  <Title level={4} className="!mt-3">
                    Automated Marking & Feedback
                  </Title>
                  <Paragraph>
                    Staff can configure automated grading flows with custom test scripts, memos, and
                    mark allocation.
                  </Paragraph>
                </Card>
              </Col>
            </Row>
          </section>

          {/* Carousel Section */}
          <section className="bg-gray-50 dark:bg-gray-900 !px-6 !py-16 !text-center">
            <Title level={3}>See It in Action</Title>
            <Paragraph className="!max-w-2xl !mx-auto !text-gray-600 dark:!text-gray-300 !mb-6">
              From assignment creation to submission tracking, every view is designed with usability
              in mind.
            </Paragraph>
            <div className="!max-w-4xl !mx-auto !rounded-lg !overflow-hidden !border !border-gray-300 dark:!border-gray-700">
              <Carousel autoplay dotPosition="bottom">
                {carouselImages.map((src, index) => (
                  <div
                    key={index}
                    className="!flex !justify-center !items-center !bg-white dark:!bg-gray-900"
                  >
                    <img
                      src={src}
                      alt={`Slide ${index + 1}`}
                      className="!w-full !h-auto !aspect-[1590/942] !object-contain"
                    />
                  </div>
                ))}
              </Carousel>
            </div>
          </section>
        </main>

        {/* Footer */}
        <footer className="!text-center !py-8 !text-sm !text-gray-400 dark:!text-gray-600">
          © {new Date().getFullYear()} FitchFork. Built for education.
        </footer>
      </div>
    </div>
  );
};

export default Landing;
