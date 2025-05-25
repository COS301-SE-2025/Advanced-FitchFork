import { Typography, Card, Row, Col } from 'antd';

const { Text } = Typography;

export default function GeneralInfoSection() {
  return (
    <div className="overflow-hidden">
      <Row gutter={[24, 24]}>
        <Col span={12}>
          <Card title="Module Overview">
            <Row gutter={[0, 12]}>
              <Col span={12}>
                <Text strong>Code</Text>
                <div>COS132</div>
              </Col>
              <Col span={12}>
                <Text strong>Year</Text>
                <div>2025</div>
              </Col>
              <Col span={24}>
                <Text strong>Description</Text>
                <div>
                  An introductory course in computer science and programming fundamentals. This
                  module helps students learn the basics of algorithms, data structures, and Python.
                </div>
              </Col>
              <Col span={12}>
                <Text strong>Created At</Text>
                <div>2025-01-10 14:23</div>
              </Col>
              <Col span={12}>
                <Text strong>Last Updated</Text>
                <div>2025-03-05 09:17</div>
              </Col>
            </Row>
          </Card>
        </Col>

        <Col span={12}>
          <Card title="People Summary">
            <Row gutter={[0, 12]}>
              <Col span={12}>
                <Text strong>Lecturers</Text>
                <div>2 assigned</div>
              </Col>
              <Col span={12}>
                <Text strong>Tutors</Text>
                <div>4 assigned</div>
              </Col>
              <Col span={24}>
                <Text strong>Students</Text>
                <div>134 enrolled</div>
              </Col>
            </Row>
          </Card>
        </Col>

        <Col span={24}>
          <Card title="Assignments Summary">
            <Row gutter={[0, 12]}>
              <Col span={12}>
                <Text strong>Total Assignments</Text>
                <div>5</div>
              </Col>
              <Col span={12}>
                <Text strong>Next Due Date</Text>
                <div>2025-05-30</div>
              </Col>
            </Row>
          </Card>
        </Col>
      </Row>
    </div>
  );
}
