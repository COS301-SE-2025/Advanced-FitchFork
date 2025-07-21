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
}

const noop = Object.assign(() => {}, {
  then: () => Promise.resolve(),
}) as unknown as MessageType;

const fallback: MessageApi = {
  success: () => noop,
  error: () => noop,
  info: () => noop,
  warning: () => noop,
  loading: () => noop,
};

export let message: MessageApi = fallback;

export const MessageContextHolder = () => {
  const [api, contextHolder] = antdMessage.useMessage();

  useEffect(() => {
    message = {
      success: (content) =>
        typeof content === 'string' ? api.success({ content }) : api.success(content),
      error: (content) =>
        typeof content === 'string' ? api.error({ content }) : api.error(content),
      info: (content) => (typeof content === 'string' ? api.info({ content }) : api.info(content)),
      warning: (content) =>
        typeof content === 'string' ? api.warning({ content }) : api.warning(content),
      loading: (content) =>
        typeof content === 'string' ? api.loading({ content }) : api.loading(content),
    };
  }, [api]);

  return <>{contextHolder}</>;
};
