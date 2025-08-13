import { Typography } from 'antd';
import clsx from 'clsx';
import { Link } from 'react-router-dom';
import { useTheme } from '@/context/ThemeContext';

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

const Logo = ({
  collapsed = false,
  className = '',
  showText = true,
  size = 'md',
  variant = 'auto',
  shadow = false,
}: LogoProps) => {
  const { isDarkMode } = useTheme();

  const { img: imgSize, text: textSize, shadow: shadowClass } = sizeMap[size];

  const logoSrc =
    variant === 'light'
      ? '/ff_logo_light.svg'
      : variant === 'dark'
        ? '/ff_logo_dark.svg'
        : isDarkMode
          ? '/ff_logo_dark.svg'
          : '/ff_logo_light.svg';

  return (
    <Link
      to="/"
      className={clsx(
        'inline-flex items-center gap-4 no-underline text-inherit',
        collapsed && 'scale-90',
        shadow && shadowClass,
        className,
      )}
    >
      <div
        className={clsx(
          'flex items-center gap-4 transition-all duration-300 ease-in-out rounded-md',
        )}
      >
        <img
          src={logoSrc}
          alt="FitchFork Logo"
          className={clsx(imgSize, 'w-auto object-contain rounded-md')}
        />

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
};

export default Logo;
