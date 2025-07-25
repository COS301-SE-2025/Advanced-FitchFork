import { Modal } from 'antd';
import React from 'react';

interface ConfirmModalProps {
  open: boolean;
  title: string;
  content?: string;
  okText?: string;
  cancelText?: string;
  onOk: () => void;
  onCancel: () => void;
}

const ConfirmModal: React.FC<ConfirmModalProps> = ({
  open,
  title,
  content = 'This action cannot be undone.',
  okText = 'Yes',
  cancelText = 'No',
  onOk,
  onCancel,
}) => {
  return (
    <Modal
      open={open}
      title={<span>{title}</span>}
      okText={okText}
      cancelText={cancelText}
      onOk={onOk}
      onCancel={onCancel}
    >
      {content}
    </Modal>
  );
};

export default ConfirmModal;
