import { Row, Col, Typography } from 'antd';
import {
  CodeOutlined,
  WarningOutlined,
  QuestionCircleOutlined,
  FireOutlined,
} from '@ant-design/icons';
import StatCard from '../StatCard';

const { Title } = Typography;

const insights = [
  {
    icon: <CodeOutlined className="!text-blue-500 text-xl" />,
    label: 'Most Active Module',
    value: 'COS 344 - Graphics',
  },
  {
    icon: <WarningOutlined className="!text-red-500 text-xl" />,
    label: 'Failing Test Coverage',
    value: '3 Assignments',
  },
  {
    icon: <QuestionCircleOutlined className="!text-yellow-500 text-xl" />,
    label: 'High Student Confusion',
    value: '5 Assignments',
  },
  {
    icon: <FireOutlined className="!text-orange-500 text-xl" />,
    label: 'Marking Failures',
    value: '4 Crashing Jobs',
  },
];

const ModuleAssignmentsPanel = () => {
  return (
    <div className="bg-white dark:bg-gray-950 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
      <Title level={4}>Module & Assignment Insights</Title>
      <Row gutter={[16, 16]}>
        {insights.map(({ icon, label, value }, index) => (
          <Col key={index} xs={24} sm={12} md={12} lg={6}>
            <StatCard title={label} value={value} prefix={icon} />
          </Col>
        ))}
      </Row>
    </div>
  );
};

export default ModuleAssignmentsPanel;
