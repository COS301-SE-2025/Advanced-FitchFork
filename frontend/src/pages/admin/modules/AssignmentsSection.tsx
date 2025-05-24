import { Typography } from 'antd';

const { Title, Text } = Typography;

export default function AssignmentsSection() {
  return (
    <div>
      <Title level={4}>Assignment List</Title>
      <Text>This tab will list and allow managing assignments for this module.</Text>
    </div>
  );
}
