import {
  Typography,
  Tag,
  Button,
  Descriptions,
  Tabs,
  Progress,
  Alert,
  Card,
  Space,
  Steps,
} from 'antd';
import {
  DownloadOutlined,
  FileDoneOutlined,
  CheckCircleTwoTone,
  WarningTwoTone,
  LoadingOutlined,
} from '@ant-design/icons';
import SubmissionTasks, { type SubmissionTask } from '@/components/submissions/SubmisisonTasks';
const { Title, Text, Paragraph } = Typography;

const steps = [
  {
    title: 'Info',
    description: 'Submission Details',
  },
  {
    title: 'Tasks',
    description: 'Breakdown of all sections',
  },
  {
    title: 'Testing',
    description: 'Code coverage results',
    icon: <LoadingOutlined spin />,
  },
  {
    title: 'Feedback',
    description: 'Evaluator remarks',
  },
];

const tasks: SubmissionTask[] = [
  {
    key: '1',
    title: '1 item',
    score: '22 / 22',
    feedback: [
      { label: 'Section 1', score: '6/6', status: 'correct' },
      { label: 'Section 2', score: '3/3', status: 'correct' },
      { label: 'Section 3', score: '11/11', status: 'correct' },
      { label: 'All memory freed', score: '2/2', status: 'correct' },
    ],
  },
  {
    key: '2',
    title: '2 products',
    score: '33 / 44',
    feedback: [
      { label: 'Section 1', score: '2/2', status: 'correct' },
      { label: 'Section 2', score: '2/5', status: 'incorrect' },
      { label: 'Section 3', score: '4/4', status: 'correct' },
      { label: 'Section 4', score: '1/1', status: 'correct' },
      { label: 'Section 5', score: '0/0', status: 'info' },
      { label: 'Section 6', score: '2/2', status: 'correct' },
      { label: 'Section 7', score: '2/5', status: 'incorrect' },
      { label: 'Section 8', score: '4/4', status: 'correct' },
      { label: 'Section 9', score: '0/2', status: 'incorrect' },
      { label: 'Section 10', score: '0/0', status: 'info' },
      { label: 'Section 11', score: '2/2', status: 'correct' },
      { label: 'Section 12', score: '3/6', status: 'incorrect' },
      { label: 'Section 13', score: '5/5', status: 'correct' },
      { label: 'Section 14', score: '2/2', status: 'correct' },
      { label: 'All memory freed', score: '4/4', status: 'correct' },
    ],
  },
  {
    key: '3',
    title: '3 services',
    score: '242 / 242',
    feedback: [
      { label: 'Section 1', score: '7/7', status: 'correct' },
      { label: 'Section 2', score: '30/30', status: 'correct' },
      { label: 'Section 3', score: '30/30', status: 'correct' },
      { label: 'Section 4', score: '6/6', status: 'correct' },
      { label: 'Section 5', score: '0/0', status: 'info' },
      { label: 'Section 6', score: '7/7', status: 'correct' },
      { label: 'Section 7', score: '30/30', status: 'correct' },
      { label: 'Section 8', score: '30/30', status: 'correct' },
      { label: 'Section 9', score: '6/6', status: 'correct' },
      { label: 'Section 10', score: '0/0', status: 'info' },
      { label: 'Section 11', score: '7/7', status: 'correct' },
      { label: 'Section 12', score: '30/30', status: 'correct' },
      { label: 'Section 13', score: '30/30', status: 'correct' },
      { label: 'Section 14', score: '7/7', status: 'correct' },
      { label: 'All memory freed', score: '22/22', status: 'correct' },
    ],
  },
  {
    key: '4',
    title: '4 shop',
    score: '40 / 44',
    feedback: [
      { label: 'Section 1', score: '1/1', status: 'correct' },
      { label: 'Section 2', score: '3/3', status: 'correct' },
      { label: 'Section 3', score: '5/6', status: 'incorrect' },
      { label: 'Section 4', score: '4/4', status: 'correct' },
      { label: 'Section 5', score: '7/7', status: 'correct' },
      { label: 'Section 6', score: '16/19', status: 'incorrect' },
      { label: 'All memory freed', score: '4/4', status: 'correct' },
    ],
  },
];

const SubmissionView = () => {
  return (
    <div className="p-4 sm:p-6 max-w-7xl">
      <div className="grid grid-cols-1 xl:grid-cols-4 gap-6">
        {/* Main Content */}
        <div className="xl:col-span-3 space-y-6">
          <div className="flex justify-between items-center flex-wrap gap-4">
            <div>
              <Title level={4} className="mb-0">
                Practical 6 - COS110 (2023)
              </Title>
              <Text type="secondary">Marking Scheme: Best mark</Text>
            </div>
            <Space wrap>
              <Button type="primary" icon={<FileDoneOutlined />}>
                New Practice Submission
              </Button>
              <Button icon={<DownloadOutlined />}>Download</Button>
            </Space>
          </div>

          <Alert
            message="Past Due Date - Practice submissions only"
            description="Practice submissions won't be considered for your final mark."
            type="warning"
            showIcon
            className="!mb-4"
          />

          <Descriptions title="Submission Info" bordered column={2} size="middle">
            <Descriptions.Item label="Mark">
              <Tag color="green">375 / 390</Tag> <Text type="secondary">(96.15%)</Text>
            </Descriptions.Item>
            <Descriptions.Item label="Attempt">
              <Tag>#4 of 13</Tag>
            </Descriptions.Item>
            <Descriptions.Item label="Uploaded At">05/10/2023 12:55:49</Descriptions.Item>
            <Descriptions.Item label="File Hash (MD5)">
              <Text code>dbcfd7aaad61537e61d95ed3e3b09e3d</Text>
            </Descriptions.Item>
          </Descriptions>

          <Tabs defaultActiveKey="tasks" size="large" type="line">
            <Tabs.TabPane tab="Tasks" key="tasks">
              <SubmissionTasks tasks={tasks} />
            </Tabs.TabPane>

            <Tabs.TabPane tab="Testing" key="testing">
              <Card
                title="Code Coverage"
                className="!border-gray-200 dark:!border-neutral-800 dark:!bg-neutral-900"
              >
                <div className="grid gap-4 sm:grid-cols-2">
                  {[
                    ['shop', 100],
                    ['item', 100],
                    ['product', 100],
                    ['service', 100],
                    ['bulk', 100],
                    ['discountedProduct', 100],
                    ['subscription', 81.25],
                    ['labor', 83.56],
                  ].map(([cls, pct]) => {
                    const percent = +pct;
                    const color = percent === 100 ? undefined : '#faad14';
                    const status = percent === 100 ? 'success' : 'active';
                    return (
                      <div
                        key={cls}
                        className="p-4 rounded-lg border border-gray-100 dark:border-neutral-700 bg-white dark:bg-neutral-800"
                      >
                        <div className="flex justify-between items-center mb-2">
                          <Text className="font-medium">{cls}</Text>
                          <Text type="secondary">{percent.toFixed(2)}%</Text>
                        </div>
                        <Progress percent={percent} status={status} strokeColor={color} />
                      </div>
                    );
                  })}
                </div>
              </Card>
            </Tabs.TabPane>

            <Tabs.TabPane tab="Feedback" key="feedback">
              <Card title="Evaluator's Feedback" bordered>
                <Paragraph>
                  <CheckCircleTwoTone twoToneColor="#52c41a" className="mr-2" />
                  This submission is well structured and passes all tests.
                </Paragraph>
                <Paragraph>
                  <WarningTwoTone twoToneColor="#faad14" className="mr-2" />
                  Consider optimizing product purchase logic in Section 2.
                </Paragraph>
              </Card>
            </Tabs.TabPane>
          </Tabs>
        </div>

        {/* Right Column: Steps */}
        <div className="hidden xl:block">
          <Steps direction="vertical" current={1} items={steps} />
        </div>
      </div>
    </div>
  );
};

export default SubmissionView;
