// frontend: MessageContextHolder.tsx
import { useEffect } from 'react';
import { message as antdMessage } from 'antd';
import type { ArgsProps } from 'antd/es/message';
import type { MessageType } from 'antd/es/message/interface';

interface MessageApi {
  success: (content: string | ArgsProps) => MessageType;
  error: (content: string | ArgsProps) => MessageType;
  info: (content: string | ArgsProps) => MessageType;
  warning: (content: string | ArgsProps) => MessageType;
  loading: (content: string | ArgsProps) => MessageType;
  notImplemented: () => MessageType;
}

const noop = Object.assign(() => {}, { then: () => Promise.resolve() }) as unknown as MessageType;

const fallback: MessageApi = {
  success: () => noop,
  error: () => noop,
  info: () => noop,
  warning: () => noop,
  loading: () => noop,
  notImplemented: () => noop,
};

export let message: MessageApi = fallback;

function withTestId(content: string | ArgsProps, testId: string): ArgsProps {
  const wrap = (node: React.ReactNode) => <span data-testid={testId}>{node}</span>;

  if (typeof content === 'string') {
    return { content: wrap(content) };
  }
  // ArgsProps
  const given = content as ArgsProps;
  const node = 'content' in given ? given.content : undefined;
  return { ...given, content: wrap(node ?? '') };
}

export const MessageContextHolder = () => {
  const [api, contextHolder] = antdMessage.useMessage();

  useEffect(() => {
    message = {
      success: (c) => api.success(withTestId(c, 'toast-success')),
      error: (c) => api.error(withTestId(c, 'toast-error')),
      info: (c) => api.info(withTestId(c, 'toast-info')),
      warning: (c) => api.warning(withTestId(c, 'toast-warning')),
      loading: (c) => api.loading(withTestId(c, 'toast-loading')),
      notImplemented: () =>
        api.info(
          withTestId(
            { content: 'This feature is not implemented yet.', duration: 2 },
            'toast-info',
          ),
        ),
    };
  }, [api]);

  return <>{contextHolder}</>;
};
