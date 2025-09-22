import { Tooltip, Typography } from 'antd';
import { QuestionCircleOutlined, InfoCircleOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';

type TipProps = {
  text?: string;
  to?: string;
  iconOnly?: boolean;
  showIcon?: boolean;
  className?: string;
  type?: 'help' | 'info';
  newTab?: boolean;
};

const { Paragraph } = Typography;

const Tip = ({
  text,
  to,
  iconOnly = false,
  showIcon = false,
  className,
  type = 'help',
  newTab = false,
}: TipProps) => {
  const navigate = useNavigate();
  const Icon = type === 'info' ? InfoCircleOutlined : QuestionCircleOutlined;

  const handleClick = (e: React.MouseEvent<HTMLElement>) => {
    if (!to) return;
    e.preventDefault();
    if (newTab) {
      window.open(to, '_blank', 'noopener,noreferrer');
    } else {
      navigate(to);
    }
  };

  // Icon-only variant (tight baseline alignment with text)
  if (iconOnly) {
    return (
      <Tooltip title={text}>
        <span
          role={to ? 'button' : undefined}
          onClick={to ? handleClick : undefined}
          className={`
            inline-block align-middle leading-none cursor-pointer
            !text-gray-500 dark:!text-gray-500
            ${className || ''}
          `}
        >
          <Icon className="relative top-[1px]" />
        </span>
      </Tooltip>
    );
  }

  // Text or icon + text variant
  return (
    <Paragraph
      onClick={to ? handleClick : undefined}
      className={`
        group m-0 p-0 inline-block
        !text-xs !text-gray-500 dark:!text-gray-400
        ${to ? 'cursor-pointer' : ''}
        ${className || ''}
      `}
    >
      <span className="inline-flex items-center gap-1">
        {showIcon && (
          <span className="inline-flex items-center !text-gray-500 dark:text-gray-500">
            <Icon />
          </span>
        )}
        <span className="group-hover:underline underline-offset-2">{text}</span>
      </span>
    </Paragraph>
  );
};

export default Tip;
