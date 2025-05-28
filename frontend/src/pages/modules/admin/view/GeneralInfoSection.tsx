import { Row, Col, Descriptions } from 'antd';
import type { ModuleDetailsResponse } from '@/types/modules';

interface Props {
  module: ModuleDetailsResponse;
}

export default function GeneralInfoSection({ module }: Props) {
  return (
    <div className="overflow-hidden">
      <Row gutter={[24, 24]}>
        <Col span={12}>
          <Descriptions title="Module Overview" bordered column={1} size="middle">
            <Descriptions.Item label="Code">{module.code}</Descriptions.Item>
            <Descriptions.Item label="Year">{module.year}</Descriptions.Item>
            <Descriptions.Item label="Description">{module.description}</Descriptions.Item>
            <Descriptions.Item label="Created At">{module.created_at}</Descriptions.Item>
            <Descriptions.Item label="Last Updated">{module.updated_at}</Descriptions.Item>
          </Descriptions>
        </Col>

        <Col span={12}>
          <Descriptions title="People Summary" bordered column={1} size="middle">
            <Descriptions.Item label="Lecturers">
              {module.lecturers.length} assigned
            </Descriptions.Item>
            <Descriptions.Item label="Tutors">{module.tutors.length} assigned</Descriptions.Item>
            <Descriptions.Item label="Students">
              {module.students.length} enrolled
            </Descriptions.Item>
          </Descriptions>
        </Col>

        <Col span={24}>
          <Descriptions title="Assignments Summary" bordered column={2} size="middle">
            <Descriptions.Item label="Total Assignments">5</Descriptions.Item>
            <Descriptions.Item label="Next Due Date">2025-05-30</Descriptions.Item>
          </Descriptions>
        </Col>
      </Row>
    </div>
  );
}
