import dayjs from 'dayjs';
import { Typography, Input, Select, DatePicker } from 'antd';

const { Title, Paragraph, Text } = Typography;
const { Option } = Select;

type Props = {
  draft: {
    name: string;
    assignment_type: string;
    available_from: string;
    due_date: string;
  };
  setDraft: (draft: any) => void;
};

const StepCreateAssignment = ({ draft, setDraft }: Props) => {
  return (
    <div className="space-y-6">
      <Title level={3}>Create Assignment</Title>
      <Paragraph type="secondary">
        Fill in the basic assignment details and click Next to create it on the server.
      </Paragraph>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div>
          <Text strong>Assignment Name</Text>
          <Input
            placeholder="e.g. Image Compression Project"
            value={draft.name}
            onChange={(e) => setDraft({ ...draft, name: e.target.value })}
            className="mt-1"
          />
        </div>
        <div>
          <Text strong>Type</Text>
          <Select
            value={draft.assignment_type}
            onChange={(v) => setDraft({ ...draft, assignment_type: v })}
            className="w-full mt-1"
          >
            <Option value="assignment">Assignment</Option>
            <Option value="quiz">Quiz</Option>
            <Option value="project">Project</Option>
          </Select>
        </div>
        <div>
          <Text strong>Available From</Text>
          <DatePicker
            className="w-full mt-1"
            showTime={{ format: 'HH:mm' }}
            format="YYYY-MM-DD HH:mm"
            value={draft.available_from ? dayjs(draft.available_from) : null}
            onChange={(val) => setDraft({ ...draft, available_from: val ? val.toISOString() : '' })}
          />
        </div>
        <div>
          <Text strong>Due Date</Text>
          <DatePicker
            className="w-full mt-1"
            showTime={{ format: 'HH:mm' }}
            format="YYYY-MM-DD HH:mm"
            value={draft.due_date ? dayjs(draft.due_date) : null}
            onChange={(val) => setDraft({ ...draft, due_date: val ? val.toISOString() : '' })}
          />
        </div>
      </div>
    </div>
  );
};

export default StepCreateAssignment;
