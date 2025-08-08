import React from 'react';
import { Button, Input, Space, Tooltip, Typography } from 'antd';
import { EditOutlined, DeleteOutlined } from '@ant-design/icons';
import ReactMarkdown from 'react-markdown';
import rehypeHighlight from 'rehype-highlight';
import dayjs from 'dayjs';
import UserAvatar from '@/components/common/UserAvatar';

const { Text } = Typography;

export interface ChatEntry {
  id: number;
  sender: string;
  content: string;
  timestamp: string;
  system?: boolean;
}

interface Props {
  message: ChatEntry;
  prevMessage?: ChatEntry;
  isOwn: boolean;
  isEditing: boolean;
  editedContent: string;
  onEditChange: (value: string) => void;
  onSave: () => void;
  onCancel: () => void;
  onStartEdit: () => void;
  onDelete: () => void;
}

const TicketChatMessage: React.FC<Props> = ({
  message,
  prevMessage,
  isOwn,
  isEditing,
  editedContent,
  onEditChange,
  onSave,
  onCancel,
  onStartEdit,
  onDelete,
}) => {
  const showAvatarAndName =
    !prevMessage ||
    prevMessage.sender !== message.sender ||
    dayjs(message.timestamp, 'HH:mm').diff(dayjs(prevMessage.timestamp, 'HH:mm'), 'minute') > 2;

  const sharedBg = isEditing
    ? 'bg-gray-100 dark:bg-gray-800'
    : 'hover:bg-gray-100 dark:hover:bg-gray-900';

  const content = (
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
                onSave();
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
              onClick={onSave}
              className="!text-blue-500 hover:underline p-0 border-0 bg-transparent"
            >
              save
            </button>
          </div>
        </div>
      ) : (
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
      )}
    </div>
  );

  if (message.system) {
    return (
      <div key={message.id} className="text-center text-gray-400 italic text-xs my-2">
        <div className="max-w-full text-sm">{content}</div>
      </div>
    );
  }

  return showAvatarAndName ? (
    <div
      key={message.id}
      className={`group flex items-start gap-2 px-3 transition-colors pt-1 mt-3 ${sharedBg}`}
    >
      <UserAvatar user={{ id: -1, username: message.sender }} className="flex-shrink-0 pt-1" />

      <div className="flex-1 relative">
        {/* Top-right edit/delete when NOT editing */}
        {!isEditing && isOwn && (
          <div className="absolute top-0 right-0 hidden group-hover:flex rounded shadow z-10">
            <Space.Compact size="small">
              <Tooltip title="Edit">
                <Button size="small" icon={<EditOutlined />} onClick={onStartEdit} />
              </Tooltip>
              <Tooltip title="Delete">
                <Button size="small" danger icon={<DeleteOutlined />} onClick={onDelete} />
              </Tooltip>
            </Space.Compact>
          </div>
        )}

        <div className="flex items-center justify-between mb-0.5">
          <div className="flex items-center gap-2">
            <Text strong className="text-sm">
              {message.sender}
            </Text>
            <Text type="secondary" className="text-xs">
              {message.timestamp}
            </Text>
          </div>
        </div>
        <div className="mb-[-2px]">{content}</div>
      </div>
    </div>
  ) : (
    <div key={message.id} className={`group w-full px-2 transition-colors ${sharedBg}`}>
      <div className="ml-[52px] relative">
        {!isEditing && isOwn && (
          <div className="absolute top-0 right-0 hidden group-hover:flex shadow z-10">
            <Space.Compact size="small">
              <Tooltip title="Edit">
                <Button size="small" icon={<EditOutlined />} onClick={onStartEdit} />
              </Tooltip>
              <Tooltip title="Delete">
                <Button size="small" danger icon={<DeleteOutlined />} onClick={onDelete} />
              </Tooltip>
            </Space.Compact>
          </div>
        )}
        {content}
      </div>
    </div>
  );
};

export default TicketChatMessage;
