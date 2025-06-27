import { Collapse, Typography, Divider } from 'antd';

const { Title, Paragraph, Text } = Typography;
const { Panel } = Collapse;

const HelpTroubleshooting = () => {
  return (
    <div>
      <Title level={3}>Troubleshooting</Title>
      <Paragraph>
        Having trouble using the platform? Below are solutions to common problems, including login
        issues, permission errors, and broken interfaces.
      </Paragraph>

      <Divider />

      <Title level={4}>Access & Login Issues</Title>
      <Collapse accordion>
        <Panel header="I see a 401 error (Unauthorized)" key="1">
          <Paragraph>
            This usually means you are not logged in. Try logging in again at{' '}
            <Text code>/login</Text>. If the issue persists, clear your browser cache and retry.
          </Paragraph>
        </Panel>

        <Panel header="I see a 403 error (Forbidden)" key="2">
          <Paragraph>
            This means you're logged in, but you don&apos;t have permission to access the page or
            feature. Check if you are assigned the correct role for the module, or contact your
            admin.
          </Paragraph>
        </Panel>

        <Panel header="I can't log in at all" key="3">
          <Paragraph>
            Double-check your email and password. If needed, click{' '}
            <Text strong>Forgot Password</Text> on the login page to reset your credentials.
          </Paragraph>
        </Panel>
      </Collapse>

      <Divider />

      <Title level={4}>UI & Navigation Problems</Title>
      <Collapse accordion>
        <Panel header="The page is blank or won't load" key="4">
          <Paragraph>
            Try refreshing the page or clearing your browser cache. If you&apos;re using extensions
            like ad blockers, try disabling them temporarily.
          </Paragraph>
        </Panel>

        <Panel header="The sidebar or layout is broken" key="5">
          <Paragraph>
            Resize the window to trigger layout adjustments, or try switching between light/dark
            mode in your profile. If that fails, log out and log back in.
          </Paragraph>
        </Panel>
      </Collapse>

      <Divider />

      <Title level={4}>Submission Issues</Title>
      <Collapse accordion>
        <Panel header="My upload isn't showing" key="6">
          <Paragraph>
            After submitting, your file should appear in the list. If not, check your internet
            connection and try again. Ensure the file size and type are allowed.
          </Paragraph>
        </Panel>

        <Panel header="I uploaded the wrong file" key="7">
          <Paragraph>
            If the deadline hasn&apos;t passed, you can re-upload. The newest file will replace the
            previous submission.
          </Paragraph>
        </Panel>
      </Collapse>

      <Divider />

      <Title level={4}>General Tips</Title>
      <ul style={{ paddingLeft: '1.5rem' }}>
        <li>
          <Text strong>Refresh the page</Text> if something looks wrong.
        </li>
        <li>
          <Text strong>Clear your browser cache</Text> to fix loading glitches.
        </li>
        <li>
          <Text strong>Use Google Chrome</Text> or Firefox for best compatibility.
        </li>
        <li>
          <Text strong>Try incognito mode</Text> if your session is behaving oddly.
        </li>
      </ul>

      <Divider />

      <Title level={4}>Still Stuck?</Title>
      <Paragraph>
        Take a screenshot of the problem and email{' '}
        <a href="mailto:support@example.com">support@example.com</a>. Include your username,
        browser, and steps to reproduce the issue.
      </Paragraph>
    </div>
  );
};

export default HelpTroubleshooting;
