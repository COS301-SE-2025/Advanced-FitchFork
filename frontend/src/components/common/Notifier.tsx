import { App as AntApp } from 'antd';

const useNotifier = () => {
  const { notification } = AntApp.useApp();

  const notifyInfo = (message: string, description?: string) => {
    notification.info({
      message,
      description,
      placement: 'bottomRight',
    });
  };

  const notifyError = (message: string, description?: string) => {
    notification.error({
      message,
      description,
      placement: 'bottomRight',
    });
  };

  const notifySuccess = (message: string, description?: string) => {
    notification.success({
      message,
      description,
      placement: 'bottomRight',
    });
  };

  return { notifyInfo, notifyError, notifySuccess };
};

export default useNotifier;
