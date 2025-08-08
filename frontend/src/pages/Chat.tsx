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

function stringToPastelHSL(username: string): string {
  const hash = Array.from(username).reduce((acc, char) => acc + char.charCodeAt(0), 0);
  const hue = hash % 360; // full hue range
  return `hsl(${hue}, 70%, 85%)`; // pastel-ish
}

function stringToDarkerPastelHSL(username: string): string {
  const hash = Array.from(username).reduce((acc, char) => acc + char.charCodeAt(0), 0);
  const hue = hash % 360;
  return `hsl(${hue}, 70%, 30%)`; // dark enough for text contrast
}

const mockMessages: ChatEntry[] = [
  {
    id: 1,
    sender: 'alice',
    content: 'Hey everyone! 👋',
    timestamp: '09:00',
  },
  {
    id: 2,
    sender: 'bob',
    content: "Hi Alice! How's your morning?",
    timestamp: '09:01',
  },
  {
    id: 3,
    sender: 'alice',
    content: "Pretty good thanks. I'm just reviewing the API docs.",
    timestamp: '09:01',
  },
  {
    id: 4,
    sender: 'alice',
    content: 'Did you see the update from Dana?',
    timestamp: '09:02',
  },
  {
    id: 5,
    sender: 'charlie',
    content: 'That sounds great. I&apos;ll test the new flow soon.',
    timestamp: '09:03',
  },
  {
    id: 6,
    sender: 'charlie',
    content: 'Let me know if you hit any weird edge cases.',
    timestamp: '09:04',
  },
  {
    id: 7,
    sender: 'dana',
    content: "**Heads up**: I'll be deploying at 10:30. Expect ~2 mins downtime.",
    timestamp: '09:06',
  },
  {
    id: 8,
    sender: 'System',
    content: 'charlie joined the chat',
    timestamp: '09:06',
    system: true,
  },
  {
    id: 9,
    sender: 'bob',
    content: '> “Make it work, make it right, make it fast.” - Kent Beck',
    timestamp: '09:08',
  },
  {
    id: 10,
    sender: 'alice',
    content: 'Check this out: [OpenAI Chat](https://chat.openai.com)',
    timestamp: '09:08',
  },
  {
    id: 11,
    sender: 'bob',
    content: 'My checklist:\n- [x] Fix header bug\n- [x] Update styles\n- [ ] Refactor auth',
    timestamp: '09:09',
  },
  {
    id: 12,
    sender: 'bob',
    content: "Can't believe how fast this week's going.",
    timestamp: '09:10',
  },
];

const Chat: React.FC = () => {
  const { user } = useAuth();
  const { isDarkMode } = useTheme();
  const [messages, setMessages] = useState<ChatEntry[]>(mockMessages);

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
      <div className="flex-1 overflow-y-auto px-4 py-4 bg-gray-50 dark:bg-gray-950">
        {messages.length === 0 ? (
          <p className="text-gray-400 text-center mt-20">No messages yet</p>
        ) : (
          messages.map((msg, index) => {
            const prevMsg = messages[index - 1];
            const showAvatarAndName =
              !prevMsg ||
              prevMsg.sender !== msg.sender ||
              dayjs(msg.timestamp, 'HH:mm').diff(dayjs(prevMsg.timestamp, 'HH:mm'), 'minute') > 2;

            const content = (
              <div className="text-sm">
                <ReactMarkdown
                  rehypePlugins={[rehypeHighlight]}
                  components={{
                    p: ({ children }) => <p className="!my-0">{children}</p>,
                    ul: ({ children }) => <ul className="my-1">{children}</ul>,
                    ol: ({ children }) => <ol className="my-1">{children}</ol>,
                    pre: ({ children }) => <pre className="my-1">{children}</pre>,
                  }}
                >
                  {msg.content}
                </ReactMarkdown>
              </div>
            );

            if (msg.system) {
              return (
                <div key={msg.id} className="text-center text-gray-400 italic text-xs my-2">
                  <div className="max-w-full text-sm">{content}</div>
                </div>
              );
            }

            return showAvatarAndName ? (
              <div key={msg.id} className="flex items-start gap-3 mt-4">
                <Avatar
                  size="large"
                  className="flex-shrink-0 pt-1"
                  style={{
                    backgroundColor: stringToPastelHSL(msg.sender),
                  }}
                >
                  <span
                    className="text-base font-semibold"
                    style={{
                      color: stringToDarkerPastelHSL(msg.sender),
                    }}
                  >
                    {msg.sender.charAt(0).toUpperCase()}
                  </span>
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
                  {content}
                </div>
              </div>
            ) : (
              <div key={msg.id} className="flex items-start gap-3">
                <div className="ml-[52px] text-sm">{content}</div>
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
