import { Typography } from 'antd';
import clsx from 'clsx';
import { Link } from 'react-router-dom'; // Import Link from React Router

const { Title } = Typography;

const sizeMap = {
  sm: {
    img: 'h-8',
    text: 'text-xl',
    shadow: 'shadow-sm',
  },
  md: {
    img: 'h-12',
    text: 'text-2xl sm:text-3xl',
    shadow: 'shadow-md',
  },
  lg: {
    img: 'h-16',
    text: 'text-3xl sm:text-4xl md:text-5xl',
    shadow: 'shadow-lg',
  },
};

interface LogoProps {
  collapsed?: boolean;
  className?: string;
  showText?: boolean;
  size?: keyof typeof sizeMap;
  variant?: 'auto' | 'light' | 'dark';
  shadow?: boolean;
}

export default function Logo({
  collapsed = false,
  className = '',
  showText = true,
  size = 'md',
  variant = 'auto',
  shadow = false,
}: LogoProps) {
  const { img: imgSize, text: textSize, shadow: shadowClass } = sizeMap[size];

  const renderLogo = () => {
    if (variant === 'light') {
      return (
        <img
          src="/ff_logo_light.svg"
          alt="FitchFork Logo (Light)"
          className={clsx(imgSize, 'w-auto object-contain rounded-lg')}
        />
      );
    }
    if (variant === 'dark') {
      return (
        <img
          src="/ff_logo_dark.svg"
          alt="FitchFork Logo (Dark)"
          className={clsx(imgSize, 'w-auto object-contain rounded-lg')}
        />
      );
    }

    return (
      <>
        <img
          src="/ff_logo_light.svg"
          alt="FitchFork Logo (Light)"
          className={clsx(imgSize, 'w-auto object-contain rounded-lg block dark:hidden')}
        />
        <img
          src="/ff_logo_dark.svg"
          alt="FitchFork Logo (Dark)"
          className={clsx(imgSize, 'w-auto object-contain rounded-lg hidden dark:block')}
        />
      </>
    );
  };

  return (
    <Link to="/" className="inline-flex items-center no-underline text-inherit">
      <div
        className={clsx(
          'flex items-center gap-4 transition-all duration-300 ease-in-out rounded-md',
          collapsed ? 'scale-90' : 'scale-100',
          shadow && shadowClass,
          className,
        )}
      >
        {renderLogo()}

        {!collapsed && showText && (
          <Title
            level={2}
            className={clsx(
              '!mb-0 font-semibold leading-tight whitespace-nowrap transition-all duration-300 ease-in-out',
              textSize,
            )}
          >
            FitchFork
          </Title>
        )}
      </div>
    </Link>
  );
}
