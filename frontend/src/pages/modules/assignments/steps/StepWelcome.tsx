import { Typography, Card } from 'antd';
const { Title, Paragraph } = Typography;

const StepWelcome = () => (
  <div className="space-y-6">
    <Title level={3}>Welcome to the Assignment Setup</Title>
    <Paragraph>
      Set up your assignment for <strong>automated grading</strong> in a few quick steps — upload
      files, define tasks, and allocate marks.
    </Paragraph>

    <Paragraph type="secondary">
      When done, students can submit, and the system takes care of grading & memos.
    </Paragraph>

    <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
      <Card>
        <Title level={5}>Step-by-Step</Title>
        <Paragraph type="secondary" className="text-sm">
          Simple, guided flow. Move back anytime.
        </Paragraph>
      </Card>

      <Card>
        <Title level={5}>Auto-Save</Title>
        <Paragraph type="secondary" className="text-sm">
          Changes are saved as you go. No stress.
        </Paragraph>
      </Card>

      <Card>
        <Title level={5}>Ready to Go</Title>
        <Paragraph type="secondary" className="text-sm">
          Finish setup and you're live for submissions.
        </Paragraph>
      </Card>
    </div>
  </div>
);

export default StepWelcome;
