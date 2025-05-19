import { Typography } from 'antd';
import clsx from 'clsx';

const { Title } = Typography;

interface LogoProps {
  size?: 'sm' | 'md' | 'lg';
  className?: string;
  showText?: boolean;
}

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

export default function Logo({ size = 'md', className = '', showText = true }: LogoProps) {
  const { img, text } = sizeMap[size];

  return (
    <div className={clsx('flex items-center gap-4', className)}>
      <img
        src="/ff_logo.svg"
        alt="FitchFork Logo"
        className={clsx(img, 'w-auto object-contain rounded-lg shadow-md')}
      />
      {showText && (
        <Title level={2} className={clsx('!mb-0 font-semibold leading-tight', text)}>
          FitchFork
        </Title>
      )}
    </div>
  );
}
