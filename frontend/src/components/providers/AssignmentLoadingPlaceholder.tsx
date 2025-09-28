import { Card, Skeleton, Space } from 'antd';

const segmentPlaceholders = Array.from({ length: 5 }, (_, index) => index);

const AssignmentLoadingPlaceholder = () => (
  <div className="h-full flex flex-col overflow-hidden">
    <div className="flex-1 overflow-y-auto px-4 pt-4 mb-4 h-full">
      <div className="flex h-full flex-col gap-4 lg:flex-row lg:items-start lg:gap-6">
        <div className="flex-1 flex flex-col gap-4 h-full">
          <Card className="mb-0" bordered>
            <Space direction="vertical" size="middle" className="w-full">
              <Space size="small" wrap>
                <Skeleton.Input active size="small" style={{ width: 220, height: 32 }} />
                <Skeleton.Button active size="small" style={{ width: 100 }} />
                <Skeleton.Button active size="small" style={{ width: 140 }} />
                <Skeleton.Button active size="small" style={{ width: 160 }} />
              </Space>

              <Skeleton active paragraph={{ rows: 2, width: ['90%', '75%'] }} title={false} />
              <Skeleton.Button active size="small" style={{ width: 180 }} />
            </Space>
          </Card>

          <Card bordered>
            <Space size="middle" wrap>
              {segmentPlaceholders.map((index) => (
                <Skeleton.Button key={index} active style={{ width: 120 }} />
              ))}
            </Space>
          </Card>

          <Card className="flex-1" bordered>
            <Space direction="vertical" size="large" className="w-full">
              <Skeleton active title paragraph={{ rows: 3, width: ['90%', '80%', '60%'] }} />
              <Skeleton
                active
                paragraph={{ rows: 4, width: ['95%', '90%', '85%', '70%'] }}
                title={false}
              />
            </Space>
          </Card>
        </div>
      </div>
    </div>
  </div>
);

export default AssignmentLoadingPlaceholder;
