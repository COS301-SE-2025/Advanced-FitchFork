import { LeftOutlined } from '@ant-design/icons';
import { useLocation, useNavigate } from 'react-router-dom';
import { useMobilePageHeader } from '@/context/MobilePageHeaderContext';
import { useMediaQuery } from 'react-responsive';

const MobilePageHeader = () => {
  const { content } = useMobilePageHeader();
  const isMobile = useMediaQuery({ maxWidth: 768 });
  const navigate = useNavigate();
  const location = useLocation();

  if (!isMobile) return null;

  const goBack = () => {
    const segments = location.pathname.split('/').filter(Boolean); // trim empty
    if (segments.length > 1) {
      const newPath = '/' + segments.slice(0, -1).join('/');
      navigate(newPath);
    } else {
      navigate('/'); // fallback to root
    }
  };

  return (
    <div className="sticky top-0 z-20 h-12 px-4 py-4 bg-white dark:bg-gray-950 border-b border-gray-200 dark:border-gray-800 flex items-center">
      <button
        onClick={goBack}
        className="flex items-center justify-center text-gray-700 dark:text-gray-200 !mr-4"
        aria-label="Back"
      >
        <LeftOutlined className="text-md leading-none" />
      </button>
      <div className="flex items-center min-w-0 flex-1 h-full">{content}</div>
    </div>
  );
};

export default MobilePageHeader;
