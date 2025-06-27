import { Collapse, Typography, Divider } from 'antd';

const { Title, Paragraph, Text } = Typography;
const { Panel } = Collapse;

const HelpAccount = () => {
  return (
    <div>
      <Title level={3}>Account & Login</Title>
      <Paragraph>
        This section covers everything related to logging in, password resets, and updating your
        personal information.
      </Paragraph>

      <Divider />

      <Title level={4}>Common Account Questions</Title>

      <Collapse accordion>
        <Panel header="How do I log in?" key="1">
          <Paragraph>
            Visit the <Text strong>Login</Text> page at <Text code>/login</Text>. Enter your email
            and password, then click <Text strong>Login</Text>.
          </Paragraph>
          <Paragraph>
            If you&apos;re a new user, go to <Text code>/signup</Text> to register an account.
          </Paragraph>
        </Panel>

        <Panel header="I forgot my password. How can I reset it?" key="2">
          <Paragraph>
            Click on <Text strong>Forgot password</Text> on the login page. Enter your email and
            submit the form. You will receive an email with a link to reset your password.
          </Paragraph>
        </Panel>

        <Panel header="How do I change my email or username?" key="3">
          <Paragraph>
            Navigate to <Text strong>Settings → Account</Text>. Here you can update your email and
            username.
          </Paragraph>
          <Paragraph>
            Make sure to click <Text strong>Save</Text> to apply your changes.
          </Paragraph>
        </Panel>

        <Panel header="How do I log out?" key="4">
          <Paragraph>
            Click your avatar or profile icon in the top right corner, then select{' '}
            <Text strong>Logout</Text> from the dropdown menu.
          </Paragraph>
        </Panel>

        <Panel header="Can I delete my account?" key="5">
          <Paragraph>
            Account deletion is not available from the interface. Please contact support at{' '}
            <a href="mailto:support@example.com">support@example.com</a> to request account removal.
          </Paragraph>
        </Panel>
      </Collapse>

      <Divider />

      <Title level={4}>Tips</Title>
      <ul style={{ paddingLeft: '1.5rem' }}>
        <li>
          <Text strong>Use a strong password:</Text> Combine uppercase, lowercase, numbers, and
          symbols.
        </li>
        <li>
          <Text strong>Don't share your login:</Text> Your account is for personal use only.
        </li>
        <li>
          <Text strong>Enable two-factor authentication:</Text> (Coming soon) for added security.
        </li>
      </ul>

      <Divider />

      <Title level={4}>Still need help?</Title>
      <Paragraph>
        Reach out to us at <a href="mailto:support@example.com">support@example.com</a> and
        we&apos;ll be happy to assist you.
      </Paragraph>
    </div>
  );
};

export default HelpAccount;
