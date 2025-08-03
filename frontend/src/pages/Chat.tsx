import React, { useEffect, useRef, useState } from 'react';
import { Input, Button, Typography, Avatar, Tag } from 'antd';
import { SendOutlined } from '@ant-design/icons';
import { useAuth } from '@/context/AuthContext';
import { WS_BASE_URL } from '@/config/api';
import { useTheme } from '@/context/ThemeContext';
import dayjs from 'dayjs';
import ReactMarkdown from 'react-markdown';
import rehypeHighlight from 'rehype-highlight';
import { message } from '@/utils/message';
import 'highlight.js/styles/github.css';
import 'highlight.js/styles/github-dark.css';

const MAX_MESSAGE_LENGTH = 500;

const { Title, Text } = Typography;

interface EventEnvelope {
  event: string;
  payload: {
    content: string;
    sender?: string;
  };
}

interface ChatEntry {
  id: number;
  sender: string;
  content: string;
  timestamp: string;
  system?: boolean;
}

let nextId = 0;

const Chat: React.FC = () => {
  const { user } = useAuth();
  const { isDarkMode } = useTheme();
  const [messages, setMessages] = useState<ChatEntry[]>([]);
  const [input, setInput] = useState('');
  const [typingUsers, setTypingUsers] = useState<Set<string>>(new Set());
  const lastTypingSentRef = useRef<number>(0);
  const wsRef = useRef<WebSocket | null>(null);
  const scrollRef = useRef<HTMLDivElement>(null);

  const token =
    typeof window !== 'undefined' ? JSON.parse(localStorage.getItem('auth') || '{}')?.token : null;

  useEffect(() => {
    if (isDarkMode) {
      import('highlight.js/styles/github-dark.css');
    } else {
      import('highlight.js/styles/github.css');
    }
  }, [isDarkMode]);

  useEffect(() => {
    const interval = setInterval(() => {
      if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
        wsRef.current.send(JSON.stringify({ type: 'ping' }));
      }
    }, 25000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    if (!token || !user) return;
    const ws = new WebSocket(`${WS_BASE_URL}/chat?token=${token}`);
    wsRef.current = ws;

    ws.onopen = () => {
      ws.send(JSON.stringify({ type: 'join', sender: user.username }));
      appendSystemMessage('You joined the chat');
    };

    ws.onmessage = (event) => {
      try {
        const msg: EventEnvelope = JSON.parse(event.data);
        const sender = msg.payload.sender || 'Anonymous';
        const content = msg.payload.content || '';

        switch (msg.event) {
          case 'chat':
            appendChatMessage(sender, content);
            break;
          case 'typing':
            handleTypingIndicator(sender);
            break;
          case 'join':
            appendSystemMessage(`${sender} joined the chat`);
            break;
        }
      } catch (err) {
        console.warn('Invalid WS payload', err);
      }
    };

    ws.onclose = () => appendSystemMessage('Connection closed');
    return () => ws.close();
  }, [token, user]);

  useEffect(() => {
    scrollRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const appendChatMessage = (sender: string, content: string) => {
    setMessages((prev) => [
      ...prev,
      { id: ++nextId, sender, content, timestamp: dayjs().format('HH:mm') },
    ]);
  };

  const appendSystemMessage = (content: string) => {
    setMessages((prev) => [
      ...prev,
      { id: ++nextId, sender: 'System', content, timestamp: dayjs().format('HH:mm'), system: true },
    ]);
  };

  const handleTypingIndicator = (sender: string) => {
    if (!sender || sender === user?.username) return;
    setTypingUsers((prev) => new Set(prev).add(sender));

    setTimeout(() => {
      setTypingUsers((prev) => {
        const next = new Set(prev);
        next.delete(sender);
        return next;
      });
    }, 2000);
  };

  const sendTyping = () => {
    const now = Date.now();
    if (!wsRef.current || wsRef.current.readyState !== WebSocket.OPEN) return;
    if (now - lastTypingSentRef.current < 1500) return;
    lastTypingSentRef.current = now;

    wsRef.current.send(
      JSON.stringify({
        type: 'typing',
        sender: user?.username || 'anonymous',
      }),
    );
  };

  const sendMessage = () => {
    if (!wsRef.current || wsRef.current.readyState !== WebSocket.OPEN) return;

    const trimmed = input.trim();
    if (!trimmed) return;

    if (trimmed.length > MAX_MESSAGE_LENGTH) {
      message.error(
        `Too many characters (${trimmed.length}). Max allowed is ${MAX_MESSAGE_LENGTH}.`,
      );
      return;
    }

    wsRef.current.send(
      JSON.stringify({
        type: 'chat',
        content: trimmed,
        sender: user?.username || 'anonymous',
      }),
    );
    setInput('');
  };

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center gap-2 p-4 bg-white dark:bg-gray-900 border-b border-gray-200 dark:border-gray-800">
        <Title level={3} className="!mb-0">
          Chat
        </Title>
        <Tag color="blue">Not persisted</Tag>
      </div>

      {/* Chat body */}
      <div className="flex-1 overflow-y-auto px-4 py-4 space-y-4 bg-gray-50 dark:bg-gray-950">
        {messages.length === 0 ? (
          <p className="text-gray-400 text-center mt-20">No messages yet</p>
        ) : (
          messages.map((msg) => {
            const content = (
              <ReactMarkdown rehypePlugins={[rehypeHighlight]}>{msg.content}</ReactMarkdown>
            );

            if (msg.system) {
              return (
                <div key={msg.id} className="text-center text-gray-400 italic text-xs my-2">
                  <div className="prose max-w-full text-sm">{content}</div>
                </div>
              );
            }

            return (
              <div key={msg.id} className="flex items-start gap-3">
                <Avatar size="large" className="flex-shrink-0 pt-1">
                  {msg.sender.charAt(0).toUpperCase()}
                </Avatar>
                <div>
                  <div className="flex items-center gap-2">
                    <Text strong className="text-sm">
                      {msg.sender}
                    </Text>
                    <Text type="secondary" className="text-xs">
                      {msg.timestamp}
                    </Text>
                  </div>
                  <div className="prose max-w-full text-sm">{content}</div>
                </div>
              </div>
            );
          })
        )}
        <div ref={scrollRef} />
      </div>

      {/* Typing indicator */}
      {typingUsers.size > 0 && (
        <div className="px-4 pb-2 text-sm italic text-gray-500 dark:text-gray-400">
          {Array.from(typingUsers).join(', ')} {typingUsers.size === 1 ? 'is' : 'are'} typing...
        </div>
      )}

      {/* Input */}
      <div className="p-4 border-t bg-white dark:bg-gray-900 border-gray-200 dark:border-gray-800">
        <div className="flex gap-2">
          <Input.TextArea
            placeholder="Message #general"
            value={input}
            autoSize={{ minRows: 1, maxRows: 6 }}
            onChange={(e) => {
              setInput(e.target.value);
              sendTyping();
            }}
            onPressEnter={(e) => {
              if (!e.shiftKey) {
                e.preventDefault();
                sendMessage();
              }
            }}
            className="rounded-l-lg"
            style={{ resize: 'none' }}
          />
          <Button
            type="primary"
            onClick={sendMessage}
            icon={<SendOutlined />}
            className="rounded-r-lg"
          >
            Send
          </Button>
        </div>
      </div>
    </div>
  );
};

export default Chat;
