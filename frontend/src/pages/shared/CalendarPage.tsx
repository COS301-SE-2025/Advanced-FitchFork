import { useMemo, useRef, useState } from 'react';
import { Calendar, Badge, Modal, Typography, Empty, Button, Space, Select, Segmented } from 'antd';
import { LeftOutlined, RightOutlined } from '@ant-design/icons';
import type { Dayjs } from 'dayjs';
import type { CalendarProps, BadgeProps, SelectProps } from 'antd';
import dayjs from 'dayjs';

type EventItem = { type: BadgeProps['status']; content: string };

const getListData = (value: Dayjs): EventItem[] => {
  switch (value.date()) {
    case 8:
      return [
        { type: 'warning', content: 'This is warning event.' },
        { type: 'success', content: 'This is usual event.' },
      ];
    case 10:
      return [
        { type: 'warning', content: 'This is warning event.' },
        { type: 'success', content: 'This is usual event.' },
        { type: 'error', content: 'This is error event.' },
      ];
    case 15:
      return [
        { type: 'warning', content: 'This is warning event' },
        { type: 'success', content: 'This is very long usual event......' },
        { type: 'error', content: 'This is error event 1.' },
        { type: 'error', content: 'This is error event 2.' },
        { type: 'error', content: 'This is error event 3.' },
        { type: 'error', content: 'This is error event 4.' },
      ];
    default:
      return [];
  }
};

const getMonthData = (value: Dayjs) => (value.month() === 8 ? 1394 : undefined);

export default function CalendarPage() {
  const [value, setValue] = useState<Dayjs>(dayjs());
  const [mode, setMode] = useState<'month' | 'year'>('month');

  const [open, setOpen] = useState(false);
  const [selectedDate, setSelectedDate] = useState<Dayjs | null>(null);
  const [selectedEvents, setSelectedEvents] = useState<EventItem[]>([]);

  const eventsKey = useMemo(() => (d: Dayjs) => d.format('YYYY-MM-DD'), []);

  // Prevent modal when navigating via header controls
  const suppressNextSelect = useRef(false);
  const guard =
    <T extends any[]>(fn: (...args: T) => void) =>
    (...args: T) => {
      suppressNextSelect.current = true;
      fn(...args);
    };

  const dateCellRender = (current: Dayjs) => {
    const listData = getListData(current);
    if (listData.length === 0) return null;

    return (
      <>
        {/* Mobile: dots only (max 3) + "+N" */}
        <ul className="flex flex-wrap gap-1 sm:hidden">
          {listData.slice(0, 3).map((item, idx) => (
            <li key={`${eventsKey(current)}-m-${idx}`}>
              <Badge status={item.type} />
            </li>
          ))}
          {listData.length > 3 && <li className="text-xs text-gray-500">+{listData.length - 3}</li>}
        </ul>

        {/* ≥ sm: dots + text */}
        <ul className="hidden sm:flex sm:flex-wrap sm:gap-1">
          {listData.map((item, idx) => (
            <li key={`${eventsKey(current)}-d-${idx}`}>
              <Badge status={item.type} text={item.content} />
            </li>
          ))}
        </ul>
      </>
    );
  };

  const monthCellRender = (current: Dayjs) => {
    const num = getMonthData(current);
    return num ? (
      <div className="notes-month text-center">
        <section className="text-xl font-semibold">{num}</section>
        <span className="text-xs text-gray-500">Backlog number</span>
      </div>
    ) : null;
  };

  const cellRender: CalendarProps<Dayjs>['cellRender'] = (current, info) => {
    if (info.type === 'date') return dateCellRender(current);
    if (info.type === 'month') return monthCellRender(current);
    return info.originNode;
  };

  const handleSelect: CalendarProps<Dayjs>['onSelect'] = (d) => {
    // ignore selects triggered by header controls
    if (suppressNextSelect.current) {
      suppressNextSelect.current = false;
      return;
    }
    const events = getListData(d);
    if (events.length === 0) return; // block empty days

    setSelectedDate(d);
    setSelectedEvents(events);
    setOpen(true);
  };

  // Build selectors (no dayjs.months() plugin needed)
  const buildMonthOptions = (base: Dayjs) =>
    Array.from({ length: 12 }, (_, i) => ({
      label: base.month(i).format('MMMM'),
      value: i,
    }));

  const buildYearOptions = (centerYear: number): SelectProps['options'] =>
    Array.from({ length: 21 }, (_, i) => {
      const y = centerYear - 10 + i;
      return { label: `${y}`, value: y };
    });

  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="flex-1 overflow-y-auto">
        <Calendar
          value={value}
          mode={mode}
          onPanelChange={(d, m) => {
            setValue(d);
            setMode(m);
          }}
          className="!p-4 sm:!p-6 !h-full"
          onChange={(d) => setValue(d)}
          onSelect={handleSelect}
          cellRender={cellRender}
          headerRender={({ value: v, type, onChange, onTypeChange }) => {
            const sync = (d: Dayjs) => {
              onChange(d);
              setValue(d);
            };

            const prev = guard(() => {
              const d = type === 'year' ? v.add(-1, 'year') : v.add(-1, 'month');
              sync(d);
            });
            const next = guard(() => {
              const d = type === 'year' ? v.add(1, 'year') : v.add(1, 'month');
              sync(d);
            });
            const today = guard(() => sync(dayjs()));

            const changeType = guard((t: 'month' | 'year') => {
              onTypeChange?.(t);
              setMode(t);
            });
            const changeYear = guard((y: number) => sync(v.year(y)));
            const changeMonth = guard((m: number) => sync(v.month(m)));

            const yearOptions = buildYearOptions(v.year());
            const monthOptions = buildMonthOptions(v);

            return (
              <>
                {/* MOBILE HEADER (default), hidden ≥ sm */}
                <div className="px-2 py-3 space-y-2 sm:hidden">
                  {/* Row 1: Prev / Today / Next (full width, evenly split) */}
                  <div className="grid grid-cols-3 gap-2 w-full">
                    <Button block icon={<LeftOutlined />} onClick={prev} aria-label="Previous" />
                    <Button block onClick={today}>
                      Today
                    </Button>
                    <Button block icon={<RightOutlined />} onClick={next} aria-label="Next" />
                  </div>

                  {/* Row 2: Year + Month selects (full width behavior) */}
                  <div className="grid grid-cols-2 gap-2 w-full">
                    <Select
                      value={v.year()}
                      options={yearOptions}
                      onChange={changeYear}
                      className={type === 'month' ? 'w-full' : 'w-full col-span-2'} // full width when month hidden
                    />
                    {type === 'month' && (
                      <Select
                        value={v.month()}
                        options={monthOptions}
                        onChange={changeMonth}
                        className="w-full"
                      />
                    )}
                  </div>

                  {/* Row 3: Segmented (Month/Year) full width */}
                  <div className="w-full">
                    <Segmented
                      block
                      value={type}
                      onChange={(val) => changeType(val as 'month' | 'year')}
                      options={[
                        { label: 'Month', value: 'month' },
                        { label: 'Year', value: 'year' },
                      ]}
                    />
                  </div>
                </div>

                {/* DESKTOP HEADER (≥ sm), hidden on mobile */}
                <div className="hidden sm:flex sm:items-center sm:justify-between px-2 py-3">
                  <Space>
                    <Button icon={<LeftOutlined />} onClick={prev} aria-label="Previous" />
                    <Button onClick={today}>Today</Button>
                    <Button icon={<RightOutlined />} onClick={next} aria-label="Next" />
                  </Space>

                  <Space>
                    <Select
                      value={v.year()}
                      options={yearOptions}
                      onChange={changeYear}
                      style={{ width: 96 }}
                    />
                    {type === 'month' && (
                      <Select
                        value={v.month()}
                        options={monthOptions}
                        onChange={changeMonth}
                        style={{ width: 132 }}
                      />
                    )}
                    <Segmented
                      value={type}
                      onChange={(val) => changeType(val as 'month' | 'year')}
                      options={[
                        { label: 'Month', value: 'month' },
                        { label: 'Year', value: 'year' },
                      ]}
                    />
                  </Space>
                </div>
              </>
            );
          }}
        />

        <Modal
          open={open}
          title={`Events on ${selectedDate?.format('dddd, DD MMM YYYY')}`}
          onCancel={() => setOpen(false)}
          footer={null}
          destroyOnClose
        >
          {selectedEvents.length === 0 ? (
            <div className="py-6">
              <Empty description="No events on this day" />
            </div>
          ) : (
            <ul className="list-none m-0 p-0 space-y-2">
              {selectedEvents.map((ev, i) => (
                <li key={`modal-${i}`} className="flex items-center gap-2">
                  <Badge status={ev.type} />
                  <Typography.Text>{ev.content}</Typography.Text>
                </li>
              ))}
            </ul>
          )}
        </Modal>
      </div>
    </div>
  );
}
