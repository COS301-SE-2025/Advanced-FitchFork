import React from 'react';
import { List, Typography, Empty } from 'antd';
import type { ListProps } from 'antd';

const { Title } = Typography;

type ListPanelProps<T> = {
  /** Default header title (used when renderHeader isn't provided) */
  title?: React.ReactNode;
  /** Right-side header content (used only with default header) */
  headerExtra?: React.ReactNode;
  /** Custom header renderer; if provided, replaces the default header completely */
  renderHeader?: () => React.ReactNode;

  /** Data + renderer (same shape as AntD List) */
  dataSource: T[];
  renderItem: ListProps<T>['renderItem'];

  /** Empty state text (optional) */
  emptyText?: React.ReactNode;

  /** Class names to tweak the outer container or list area */
  className?: string; // container
  listClassName?: string; // inner list

  /** Inline styles if needed */
  style?: React.CSSProperties;
  listStyle?: React.CSSProperties;

  /** Anything else to pass through to the underlying List */
  listProps?: Omit<ListProps<T>, 'dataSource' | 'renderItem' | 'header'>;
};

/**
 * Fixed header + scrollable List wrapper.
 * - Keeps your exact spacing and border styles
 * - Dark-mode friendly (uses the same bg/border classes you already use)
 */
export function ListPanel<T>({
  title,
  headerExtra,
  renderHeader,
  dataSource,
  renderItem,
  emptyText,
  className,
  listClassName,
  style,
  listStyle,
  listProps,
}: ListPanelProps<T>) {
  return (
    <div
      className={
        'h-full min-h-0 flex flex-col w-full bg-white dark:bg-gray-900 rounded-md border border-gray-200 dark:border-gray-800 ' +
        (className ?? '')
      }
      style={style}
    >
      {/* Fixed header (outside the scroll area) */}
      <div className="px-4 py-3 border-b border-gray-200 dark:border-gray-800">
        {renderHeader ? (
          renderHeader()
        ) : (
          <div className="flex items-center justify-between gap-2">
            {title ? (
              <Title level={5} className="!mb-0">
                {title}
              </Title>
            ) : (
              <span />
            )}
            {headerExtra}
          </div>
        )}
      </div>

      {/* Scrollable list */}
      <List
        className={'flex-1 overflow-y-auto ' + (listClassName ?? '')}
        style={listStyle}
        locale={{
          emptyText: emptyText ?? <Empty description="Nothing here yet." />,
        }}
        dataSource={dataSource}
        renderItem={renderItem}
        {...listProps}
      />
    </div>
  );
}

export default ListPanel;
