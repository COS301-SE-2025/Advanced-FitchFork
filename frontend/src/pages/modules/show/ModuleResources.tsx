import { List, Typography, Button } from 'antd';
import { DownloadOutlined, FileOutlined } from '@ant-design/icons';

const { Title, Text } = Typography;

const resources = [
  {
    id: 1,
    name: 'Lecture Notes Week 1',
    description: 'Introduction to the course and module structure.',
    fileType: 'PDF',
  },
  {
    id: 2,
    name: 'Assignment Guidelines',
    description: 'Instructions for completing practical assignments.',
    fileType: 'DOCX',
  },
  {
    id: 3,
    name: 'Reading Material',
    description: 'Suggested reading on software engineering practices.',
    fileType: 'PDF',
  },
];

const ModuleResources = () => {
  return (
    <div className="space-y-6 p-4 sm:p-6">
      <Title level={3} className="!mb-0">
        Resources
      </Title>

      <List
        itemLayout="vertical"
        dataSource={resources}
        renderItem={(item) => (
          <List.Item className="bg-white dark:bg-neutral-900 rounded-md px-4 py-3">
            <List.Item.Meta
              avatar={<FileOutlined className="text-lg text-blue-500 mt-1" />}
              title={
                <div className="flex items-center justify-between">
                  <Text strong>{item.name}</Text>
                  <Button
                    type="text"
                    icon={<DownloadOutlined />}
                    onClick={() => console.log('Download', item.name)}
                  >
                    Download
                  </Button>
                </div>
              }
              description={<Text type="secondary">{item.description}</Text>}
            />
          </List.Item>
        )}
      />
    </div>
  );
};

export default ModuleResources;
