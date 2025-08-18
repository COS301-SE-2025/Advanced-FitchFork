import React, { useCallback, useEffect, useRef, useState } from 'react';
import { Input, Button, Result, Typography, Alert } from 'antd';
import type { TextAreaRef } from 'antd/es/input/TextArea';
import { SendOutlined, LockOutlined } from '@ant-design/icons';
import { useAuth } from '@/context/AuthContext';
import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';
import { message as toast } from '@/utils/message';

import TicketChatMessage from '@/components/tickets/TicketChatMessage';
import type { ChatEntry } from '@/hooks/tickets/useTicketChat';
import { useTicketChat } from '@/hooks/tickets/useTicketChat';
import type { Ticket } from '@/types/modules/assignments/tickets';

const { Text } = Typography;
const MAX_MESSAGE_LENGTH = 500;

/* --------- Props --------- */
type TicketChatProps = {
  ticket: Ticket; // REQUIRED
};

/* --------- Memoized list --------- */
type MessageListProps = {
  messages: ChatEntry[];
  username?: string | null;
  editingMessageId: number | null;
  editedContent: string;
  onEditChange: (v: string) => void;
  onSave: (id: number, content: string) => void;
  onCancel: () => void;
  onStartEdit: (id: number, content: string) => void;
  onDelete: (id: number) => void;
  onEmptyAction?: () => void;
  showEmpty?: boolean; // <-- NEW: render empty state only after data is loaded
  isClosed: boolean;
};

const MemoMessage = React.memo(TicketChatMessage);

const MessageList = React.memo(function MessageList({
  messages,
  username,
  editingMessageId,
  editedContent,
  onEditChange,
  onSave,
  onCancel,
  onStartEdit,
  onDelete,
  onEmptyAction,
  showEmpty,
  isClosed,
}: MessageListProps) {
  if (messages.length === 0) {
    if (!showEmpty) return null; // avoid premature flash

    return (
      <div className="h-full min-h-[260px] flex items-center justify-center px-4">
        {isClosed ? (
          <Result
            status="warning"
            icon={<LockOutlined style={{ color: 'var(--ant-color-warning-text)' }} />}
            title="No messages yet"
            subTitle={
              <div>
                <div>This ticket is closed. You can view history, but cannot send messages.</div>
                <div style={{ marginTop: 8 }}>
                  <Text type="secondary" style={{ fontSize: 12 }}>
                    Reopen the ticket to continue the conversation.
                  </Text>
                </div>
              </div>
            }
          />
        ) : (
          <Result
            icon={<SendOutlined style={{ color: 'var(--ant-color-text-tertiary, #bfbfbf)' }} />}
            title="Start the conversation"
            subTitle={
              <div>
                <div>No messages yet. Be the first to send one.</div>
                <div style={{ marginTop: 8 }}>
                  <Text type="secondary" style={{ fontSize: 12 }}>
                    Press <Text keyboard>Enter</Text> to send • <Text keyboard>Shift</Text> +{' '}
                    <Text keyboard>Enter</Text> for a new line
                  </Text>
                </div>
              </div>
            }
            extra={
              <Button type="primary" onClick={onEmptyAction} icon={<SendOutlined />}>
                Start typing
              </Button>
            }
          />
        )}
      </div>
    );
  }

  return (
    <>
      {messages.map((msg, idx) => {
        const prevMsg = messages[idx - 1];
        const isOwn = msg.sender === username;
        const isEditing = editingMessageId === msg.id;
        return (
          <MemoMessage
            key={msg.id}
            message={msg}
            prevMessage={prevMsg}
            isOwn={isOwn}
            isClosed={isClosed}
            isEditing={isEditing}
            editedContent={editedContent}
            onEditChange={onEditChange}
            onSave={() => onSave(msg.id, editedContent)}
            onCancel={onCancel}
            onStartEdit={() => onStartEdit(msg.id, msg.content)}
            onDelete={() => onDelete(msg.id)}
          />
        );
      })}
    </>
  );
});

/* ---------------- Main ---------------- */
const TicketChat: React.FC<TicketChatProps> = ({ ticket }) => {
  const { user } = useAuth();
  const module = useModule();
  const { assignment } = useAssignment();

  const token =
    typeof window !== 'undefined'
      ? (JSON.parse(localStorage.getItem('auth') || '{}')?.token ?? null)
      : null;

  const { canUse, messages, send, update, remove, emitTyping, typingText, loaded } = useTicketChat({
    moduleId: module?.id ?? null,
    assignmentId: assignment?.id ?? null,
    ticketId: ticket.id,
    token,
    username: user?.username ?? null,
  });

  const isClosed = ticket.status === 'closed';

  // --- state ---
  const [input, setInput] = useState('');
  const [editingMessageId, setEditingMessageId] = useState<number | null>(null);
  const [editedContent, setEditedContent] = useState('');

  const listRef = useRef<HTMLDivElement>(null);
  const atBottomRef = useRef(true);
  const initialScrolledRef = useRef(false);
  const composerRef = useRef<TextAreaRef>(null);

  const focusComposer = useCallback(() => {
    if (!isClosed) requestAnimationFrame(() => composerRef.current?.focus?.());
  }, [isClosed]);

  const isNearBottom = (el: HTMLDivElement, threshold = 64) =>
    el.scrollHeight - el.scrollTop - el.clientHeight <= threshold;

  const scrollToBottom = (behavior: ScrollBehavior = 'auto') => {
    listRef.current?.scrollTo({ top: listRef.current.scrollHeight, behavior });
  };

  const handleScroll = useCallback(() => {
    if (listRef.current) atBottomRef.current = isNearBottom(listRef.current);
  }, []);

  useEffect(() => {
    if (!initialScrolledRef.current && listRef.current) {
      if (listRef.current.scrollHeight > listRef.current.clientHeight) {
        scrollToBottom('auto');
      }
      initialScrolledRef.current = true;
    }
  }, []);

  useEffect(() => {
    if (!listRef.current || messages.length === 0) return;
    const last = messages[messages.length - 1];
    const lastIsOwn = last.sender === user?.username;
    if (lastIsOwn || atBottomRef.current) {
      scrollToBottom(lastIsOwn ? 'smooth' : 'auto');
    }
  }, [messages, user?.username]);

  // handlers
  const handleEditChange = useCallback((v: string) => setEditedContent(v), []);
  const handleCancel = useCallback(() => setEditingMessageId(null), []);
  const handleStartEdit = useCallback((id: number, content: string) => {
    setEditingMessageId(id);
    setEditedContent(content);
  }, []);
  const handleSave = useCallback(
    async (id: number, content: string) => {
      if (!canUse) return;
      const trimmed = content.trim();
      if (!trimmed) return;
      await update(id, trimmed);
      setEditingMessageId(null);
    },
    [canUse, update],
  );
  const handleDelete = useCallback(async (id: number) => remove(id), [remove]);

  const sendMessage = useCallback(async () => {
    if (!canUse || isClosed) {
      if (isClosed) toast.info('This ticket is closed. Messaging is disabled.');
      return;
    }
    const trimmed = input.trim();
    if (!trimmed) return;
    if (trimmed.length > MAX_MESSAGE_LENGTH) {
      toast.error(`Too many characters (${trimmed.length}). Max is ${MAX_MESSAGE_LENGTH}.`);
      return;
    }
    await send(trimmed);
    setInput('');
    requestAnimationFrame(() => scrollToBottom('smooth'));
  }, [canUse, isClosed, input, send]);

  if (!module?.id || !assignment?.id) return null;

  return (
    <div className="flex flex-col flex-1 min-h-0 overflow-hidden">
      {/* Messages */}
      <div
        ref={listRef}
        onScroll={handleScroll}
        className="flex-1 min-h-0 overflow-y-auto bg-gray-50 dark:bg-gray-950 pb-3"
      >
        <MessageList
          messages={messages}
          username={user?.username}
          editingMessageId={editingMessageId}
          editedContent={editedContent}
          onEditChange={handleEditChange}
          onSave={handleSave}
          onCancel={handleCancel}
          onStartEdit={handleStartEdit}
          onDelete={handleDelete}
          onEmptyAction={focusComposer}
          showEmpty={loaded}
          isClosed={isClosed}
        />
      </div>

      {/* Composer + typing/closed indicator */}
      <div className="shrink-0">
        <div className="px-3">
          {isClosed && (
            <div className="mb-2">
              <Alert
                type="warning"
                showIcon
                message="This ticket is closed"
                description="You can still read history, but sending new messages is disabled."
              />
            </div>
          )}

          <div
            className={[
              'rounded-2xl ring-1 ring-gray-200 dark:ring-gray-800 bg-white dark:bg-gray-900 shadow-sm',
              isClosed ? 'opacity-60' : '',
            ].join(' ')}
          >
            <div className="p-2 flex gap-2 items-end">
              <Input.TextArea
                ref={composerRef}
                placeholder={isClosed ? 'Ticket is closed' : 'Message #ticket'}
                value={input}
                disabled={isClosed}
                autoSize={{ minRows: 1, maxRows: 20 }}
                onChange={(e) => {
                  if (!isClosed) {
                    setInput(e.target.value);
                    emitTyping();
                  }
                }}
                onPressEnter={(e) => {
                  if (!e.shiftKey) {
                    e.preventDefault();
                    sendMessage();
                  }
                }}
                className="flex-1 resize-none !border-0 !shadow-none !focus:ring-0 !focus:outline-none bg-transparent"
              />
              <Button
                type="primary"
                icon={isClosed ? <LockOutlined /> : <SendOutlined />}
                onClick={sendMessage}
                disabled={isClosed}
              >
                {isClosed ? 'Locked' : 'Send'}
              </Button>
            </div>
          </div>

          <div className="pl-5 my-1 h-4 text-[11px] text-gray-500 dark:text-gray-400 whitespace-nowrap overflow-hidden text-ellipsis flex items-center">
            {isClosed ? (
              <span className="flex items-center gap-1">
                <LockOutlined /> Ticket is closed — messaging disabled
              </span>
            ) : (
              <span className={typingText ? 'visible' : 'invisible'}>
                {typingText || 'placeholder'}
              </span>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export default TicketChat;
