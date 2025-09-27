import React, { useEffect, useState, useCallback } from 'react';
import { Modal, Space, InputNumber, message } from 'antd';
import {
  getMaxConcurrent,
  setMaxConcurrent as setMaxConcurrentApi,
} from '@/services/system/code_manager';

type Props = {
  open: boolean;
  onClose: () => void;
  /** optional: called after a successful save (with the new value) */
  onSaved?: (next: number) => void;
};

const CapacityModal: React.FC<Props> = ({ open, onClose, onSaved }) => {
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [effectiveMax, setEffectiveMax] = useState<number | null>(null);
  const [draft, setDraft] = useState<number | undefined>(undefined);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const res = await getMaxConcurrent();
      if (res.success && typeof res.data === 'number') {
        setEffectiveMax(res.data);
        setDraft(res.data);
      } else {
        throw new Error(res.message || 'Failed to load max concurrency');
      }
    } catch (e: any) {
      message.error(e?.message ?? 'Failed to load max concurrency');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (!open) return;
    void load();
  }, [open, load]);

  const valid = typeof draft === 'number' && draft > 0;

  const handleOk = useCallback(async () => {
    if (!valid || typeof draft !== 'number') return;
    setSaving(true);
    try {
      const res = await setMaxConcurrentApi(draft);
      if (!res.success) throw new Error(res.message || 'Update failed');
      const newVal = typeof res.data === 'number' ? res.data : draft;
      message.success('Updated code manager capacity');
      onSaved?.(newVal);
      onClose();
    } catch (e: any) {
      message.error(e?.message ?? 'Failed to update capacity');
    } finally {
      setSaving(false);
    }
  }, [valid, draft, onSaved, onClose]);

  return (
    <Modal
      title="Update code manager capacity"
      open={open}
      onCancel={onClose}
      onOk={handleOk}
      okText="Save"
      okButtonProps={{ disabled: !valid }}
      confirmLoading={saving}
      destroyOnHidden
    >
      <Space direction="vertical" size={12} className="w-full">
        <div className="text-sm text-gray-600 dark:text-gray-300">
          Choose the maximum number of code runs that can execute concurrently.
        </div>
        <InputNumber
          min={1}
          style={{ width: '100%' }}
          value={draft}
          onChange={(v) => setDraft(typeof v === 'number' ? v : undefined)}
          disabled={loading || saving}
        />
        <div className="text-xs text-gray-500 dark:text-gray-400">
          Current effective limit: {typeof effectiveMax === 'number' ? effectiveMax : '—'}
        </div>
      </Space>
    </Modal>
  );
};

export default CapacityModal;
