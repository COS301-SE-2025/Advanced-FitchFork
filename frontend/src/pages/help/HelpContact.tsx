import { Typography, Collapse, Divider } from 'antd';

const { Title, Paragraph, Text } = Typography;
const { Panel } = Collapse;

const HelpContact = () => {
  return (
    <div>
      <Title level={3}>Contact Support</Title>
      <Paragraph>
        We're here to help! If you're experiencing issues, need clarification, or want to report a
        bug, follow the guidance below to reach us efficiently.
      </Paragraph>

      <Divider />

      <Title level={4}>When to Contact Us</Title>
      <Collapse accordion>
        <Panel header="I can't log in or reset my password" key="1">
          <Paragraph>
            If the password reset link doesn&apos;t work or you no longer have access to your email,
            contact us to manually verify your identity and reset your account.
          </Paragraph>
        </Panel>

        <Panel header="Something looks broken or incorrect" key="2">
          <Paragraph>
            If a page isn&apos;t loading correctly, a button does nothing, or a grade is missing,
            let us know. Include a screenshot and a short description of what went wrong.
          </Paragraph>
        </Panel>

        <Panel header="I need to report a bug or technical error" key="3">
          <Paragraph>Please provide:</Paragraph>
          <ul style={{ paddingLeft: '1.5rem' }}>
            <li>Your username or email</li>
            <li>Steps to reproduce the issue</li>
            <li>Browser and device used</li>
            <li>Any error messages you saw</li>
          </ul>
        </Panel>

        <Panel header="I need to delete or transfer my account" key="4">
          <Paragraph>
            For account removal, data export, or role transfers (e.g. handing a module over to
            another lecturer), please make a formal request via email with your full name and
            purpose.
          </Paragraph>
        </Panel>
      </Collapse>

      <Divider />

      <Title level={4}>How to Reach Us</Title>
      <Paragraph>
        You can email us directly at: <a href="mailto:support@example.com">support@example.com</a>
      </Paragraph>
      <Paragraph>
        We typically respond within <Text strong>24-48 hours</Text> during the academic calendar.
      </Paragraph>

      <Divider />

      <Title level={4}>Tips for a Faster Response</Title>
      <ul style={{ paddingLeft: '1.5rem' }}>
        <li>
          <Text strong>Use your registered email address</Text> when contacting us.
        </li>
        <li>
          <Text strong>Be specific:</Text> Include URLs, error messages, and screenshots where
          applicable.
        </li>
        <li>
          <Text strong>Check the Help Center:</Text> The solution to your problem may already be
          listed.
        </li>
      </ul>

      <Divider />

      <Title level={4}>Emergency or Critical Support</Title>
      <Paragraph>
        If you are blocked from submitting an assignment near the deadline, use the subject line{' '}
        <Text code>[URGENT]</Text> and explain your situation clearly. We prioritize critical
        academic support during active submission windows.
      </Paragraph>
    </div>
  );
};

export default HelpContact;
