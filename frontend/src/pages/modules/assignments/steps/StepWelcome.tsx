import { Typography, Card } from 'antd';
const { Title, Paragraph } = Typography;

const StepWelcome = () => (
  <div className="space-y-6">
    <Title level={3}>Welcome to the Assignment Setup Wizard</Title>
    <Paragraph type="secondary">
      This guided flow will help you prepare your assignment for automated evaluation. You'll be
      asked to provide config details, files, tasks, and marking breakdowns.
    </Paragraph>
    <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
      <Card bordered>
        <Title level={4}>Step-by-Step</Title>
        <Paragraph type="secondary" className="text-sm">
          Go through each stage one by one. You can revisit steps at any time.
        </Paragraph>
      </Card>
      <Card bordered>
        <Title level={4}>Easy Inputs</Title>
        <Paragraph type="secondary" className="text-sm">
          We’ve broken everything into manageable pieces with simple inputs.
        </Paragraph>
      </Card>
      <Card bordered>
        <Title level={4}>Save Automatically</Title>
        <Paragraph type="secondary" className="text-sm">
          Data is saved automatically as you progress, no need to worry.
        </Paragraph>
      </Card>
    </div>
  </div>
);

export default StepWelcome;
