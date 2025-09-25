// src/components/tickets/TicketChatMessage.tsx
import React from 'react';
import { Input, Popconfirm, Tooltip, Typography } from 'antd';
import { EditOutlined, DeleteOutlined } from '@ant-design/icons';
import ReactMarkdown from 'react-markdown';
import rehypeHighlight from 'rehype-highlight';
import dayjs from 'dayjs';
import UserAvatar from '@/components/common/UserAvatar';
import type { ChatEntry } from '@/types/modules/assignments/tickets';

const { Text } = Typography;

interface Props {
  message: ChatEntry;
  prevMessage?: ChatEntry;
  isOwn: boolean;
  isClosed: boolean;
  isEditing: boolean;
  editedContent: string;
  onEditChange: (value: string) => void;
  onSave: () => void;
  onCancel: () => void;
  onStartEdit: () => void;
  onDelete: () => void;
}

type IconBtnProps = {
  onClick: () => void;
  title: string;
  danger?: boolean;
  confirm?: boolean;
  children: React.ReactNode;
};

const IconBtn = ({ onClick, title, danger, confirm, children }: IconBtnProps) => {
  const btn = (
    <button
      type="button"
      onClick={!confirm ? onClick : undefined}
      className={[
        'h-7 w-7 grid place-items-center rounded transition-colors',
        'text-gray-500 dark:text-gray-400',
        danger
          ? 'hover:text-red-500 focus:text-red-500'
          : 'hover:text-gray-700 dark:hover:text-gray-200',
        'focus:outline-none focus:ring-1 focus:ring-offset-1 focus:ring-blue-500/40',
      ].join(' ')}
      aria-label={title}
    >
      <span className="text-[15px]" style={{ color: 'inherit', lineHeight: 0 }}>
        {children}
      </span>
    </button>
  );

  return confirm ? (
    <Popconfirm
      title="Delete message?"
      description="This action cannot be undone."
      okText="Delete"
      okType="danger"
      cancelText="Cancel"
      onConfirm={onClick}
    >
      {btn}
    </Popconfirm>
  ) : (
    <Tooltip title={title}>{btn}</Tooltip>
  );
};

// normalize whitespace when checking for “unchanged”
const norm = (s: string) => s.replace(/\r\n/g, '\n').trim();

const TicketChatMessage: React.FC<Props> = ({
  message,
  prevMessage,
  isOwn,
  isClosed,
  isEditing,
  editedContent,
  onEditChange,
  onSave,
  onCancel,
  onStartEdit,
  onDelete,
}) => {
  const created = dayjs(message.createdAt);
  const updated = dayjs(message.updatedAt);
  const showEdited = updated.isAfter(created, 'second');

  const prevCreated = prevMessage ? dayjs(prevMessage.createdAt) : null;
  const showAvatarAndName =
    !prevMessage ||
    prevMessage.sender !== message.sender ||
    (prevCreated && created.diff(prevCreated, 'minute') > 2);

  const sharedBg = isEditing
    ? 'bg-gray-100 dark:bg-gray-800'
    : 'hover:bg-gray-100 dark:hover:bg-gray-900';

  const unchanged = norm(editedContent) === norm(message.content);

  const ContentBlock = (
    <div className="relative text-sm py-1 rounded-md transition-colors">
      {isEditing ? (
        <div className="flex flex-col gap-1">
          <Input.TextArea
            autoSize
            className="!text-sm"
            value={editedContent}
            onChange={(e) => onEditChange(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                if (!unchanged) onSave();
              } else if (e.key === 'Escape') {
                e.preventDefault();
                onCancel();
              }
            }}
          />
          <div className="ml-1 text-xs text-gray-500 dark:text-gray-400 mt-1">
            escape to{' '}
            <button
              onClick={onCancel}
              className="!text-blue-500 hover:underline p-0 border-0 bg-transparent"
            >
              cancel
            </button>{' '}
            • enter to{' '}
            <button
              onClick={() => {
                if (!unchanged) onSave();
              }}
              disabled={unchanged}
              className={[
                'p-0 border-0 bg-transparent',
                unchanged ? 'text-gray-400 cursor-not-allowed' : '!text-blue-500 hover:underline',
              ].join(' ')}
              title={unchanged ? 'No changes to save' : 'Save'}
            >
              save
            </button>
            {unchanged && <span className="ml-2 italic opacity-80">(no changes)</span>}
          </div>
        </div>
      ) : (
        <div className="inline-flex flex-wrap items-baseline gap-1">
          <ReactMarkdown
            rehypePlugins={[rehypeHighlight]}
            components={{
              p: ({ children }) => <p className="!my-0">{children}</p>,
              ul: ({ children }) => <ul className="my-1">{children}</ul>,
              ol: ({ children }) => <ol className="my-1">{children}</ol>,
              pre: ({ children }) => <pre className="my-1">{children}</pre>,
            }}
          >
            {message.content}
          </ReactMarkdown>
          {showEdited && (
            <span
              className="text-[11px] italic text-gray-500 dark:text-gray-400 opacity-80"
              title={`Edited ${updated.format('YYYY-MM-DD HH:mm')}`}
            >
              (edited)
            </span>
          )}
        </div>
      )}
    </div>
  );

  if (message.system) {
    return (
      <div key={message.id} className="text-center text-gray-400 italic text-xs my-2">
        <div className="max-w-full text-sm">{ContentBlock}</div>
      </div>
    );
  }

  const timeStr = created.format('HH:mm');

  return showAvatarAndName ? (
    <div
      key={message.id}
      className={`group flex items-start gap-2 px-3 transition-colors pt-1 mt-3 ${sharedBg}`}
    >
      <UserAvatar
        user={{
          id: message.user?.id ?? -1,
          username: message.sender ?? 'Unknown',
        }}
        className="flex-shrink-0 pt-1"
      />

      <div className="flex-1 relative">
        {/* Header row with name, time, and always-right icons */}
        <div className="flex items-center justify-between mb-0.5">
          <div className="flex items-center gap-2">
            <Text strong className="text-sm">
              {message.sender ?? 'Unknown'}
            </Text>
            <Text type="secondary" className="text-xs">
              {timeStr}
            </Text>
          </div>

          {!isClosed && !isEditing && isOwn && (
            <div className="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
              <IconBtn title="Edit" onClick={onStartEdit}>
                <EditOutlined />
              </IconBtn>
              <IconBtn title="Delete" onClick={onDelete} danger confirm>
                <DeleteOutlined />
              </IconBtn>
            </div>
          )}
        </div>

        <div className="mb-[-2px]">{ContentBlock}</div>
      </div>
    </div>
  ) : (
    <div key={message.id} className={`group w-full px-2 transition-colors ${sharedBg}`}>
      <div className="ml-[52px] relative">
        {/* No header row, but keep icons top-right */}
        {!isClosed && !isEditing && isOwn && (
          <div className="absolute -top-1 right-0 z-10 flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
            <IconBtn title="Edit" onClick={onStartEdit}>
              <EditOutlined />
            </IconBtn>
            <IconBtn title="Delete" onClick={onDelete} danger confirm>
              <DeleteOutlined />
            </IconBtn>
          </div>
        )}

        {ContentBlock}
      </div>
    </div>
  );
};

export default TicketChatMessage;
