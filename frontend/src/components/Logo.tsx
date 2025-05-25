import { Typography } from 'antd';
import clsx from 'clsx';

const { Title } = Typography;

const sizeMap = {
  sm: {
    img: 'h-8',
    text: 'text-xl',
  },
  md: {
    img: 'h-12',
    text: 'text-2xl sm:text-3xl',
  },
  lg: {
    img: 'h-16',
    text: 'text-3xl sm:text-4xl md:text-5xl',
  },
};

interface LogoProps {
  collapsed?: boolean;
  className?: string;
  showText?: boolean;
  size?: keyof typeof sizeMap;
  variant?: 'auto' | 'light' | 'dark'; // NEW
}

export default function Logo({
  collapsed = false,
  className = '',
  showText = true,
  size = 'md',
  variant = 'auto',
}: LogoProps) {
  const { img: imgSize, text: textSize } = sizeMap[size];

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

    // auto: light by default, dark when `.dark` class is active
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
    <div
      className={clsx(
        'flex items-center gap-4 transition-all duration-300 ease-in-out',
        collapsed ? 'scale-90' : 'scale-100',
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
  );
}
