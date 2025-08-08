import { useNotifier } from '@/components/common/Notifier';

const useNotImplemented = () => {
  const { notifyInfo } = useNotifier();

  return () => {
    notifyInfo('Not implemented', 'This feature is under construction.');
  };
};

export default useNotImplemented;
