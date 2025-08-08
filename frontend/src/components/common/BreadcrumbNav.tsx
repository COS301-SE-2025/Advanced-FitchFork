import { Breadcrumb, Button } from 'antd';
import { useNavigate } from 'react-router-dom';
import { useBreadcrumbs } from '@/hooks/useBreadcrumbs';
import { useMediaQuery } from 'react-responsive';
import classNames from 'classnames';
import { LeftOutlined } from '@ant-design/icons';

type BreadcrumbNavProps = {
  className?: string;
  showBackButton?: boolean;
};

const BreadcrumbNav = ({ className, showBackButton = false }: BreadcrumbNavProps) => {
  const navigate = useNavigate();
  const breadcrumbs = useBreadcrumbs();
  const isMobile = useMediaQuery({ maxWidth: 768 });

  const shouldCollapse = isMobile && breadcrumbs.length > 4;

  const displayedBreadcrumbs = shouldCollapse
    ? [
        breadcrumbs[0],
        { path: '', label: '...', isLast: false },
        breadcrumbs[breadcrumbs.length - 2],
        breadcrumbs[breadcrumbs.length - 1],
      ]
    : breadcrumbs;

  return (
    <div
      className={classNames(
        'flex items-center gap-2 overflow-x-auto whitespace-nowrap scrollbar-hide',
        className,
      )}
      style={{ WebkitOverflowScrolling: 'touch' }}
    >
      {isMobile && showBackButton && (
        <Button
          type="text"
          icon={<LeftOutlined />}
          onClick={() => navigate(-1)}
          className="text-gray-700 dark:text-gray-200 px-2 shrink-0"
        />
      )}

      <Breadcrumb
        separator=">"
        className="flex-nowrap flex items-center !mb-0"
        style={{ flexWrap: 'nowrap' }}
        items={displayedBreadcrumbs.map(({ path, label, isLast }) => ({
          title:
            isLast || label === '...' ? (
              <span className="inline-block align-middle">{label}</span>
            ) : (
              <a
                onClick={() => path && navigate(path)}
                className="text-blue-600 hover:underline inline-block align-middle"
              >
                {label}
              </a>
            ),
        }))}
      />
    </div>
  );
};

export default BreadcrumbNav;
