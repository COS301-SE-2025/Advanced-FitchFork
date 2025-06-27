import { Collapse, Layout, Typography, Anchor, Divider } from 'antd';

const { Title, Paragraph } = Typography;
const { Content, Sider } = Layout;
const { Panel } = Collapse;

const HelpPage = () => {
  return (
    <Layout className="p-6 bg-white min-h-screen">
      <Content style={{ paddingRight: '24px', maxWidth: '900px' }}>
        <Title level={2}>Help & Support</Title>
        <Paragraph>
          Welcome to the help center. Use the sidebar on the right to quickly jump to a section.
        </Paragraph>

        <Divider />

        <Title level={4} id="account">
          Account & Login
        </Title>
        <Collapse accordion>
          <Panel header="How do I reset my password?" key="account-1">
            <Paragraph>
              Go to the <strong>Login</strong> page, click <em>Forgot password</em>, enter your
              email, and follow the instructions.
            </Paragraph>
          </Panel>
          <Panel header="How do I change my email or username?" key="account-2">
            <Paragraph>
              Visit the <strong>Settings &gt; Account</strong> page and update your account
              information.
            </Paragraph>
          </Panel>
        </Collapse>

        <Divider />

        <Title level={4} id="assignments">
          Assignments
        </Title>
        <Collapse accordion>
          <Panel header="Where do I find my assignments?" key="assignments-1">
            <Paragraph>
              Navigate to your <strong>Modules</strong> and open the relevant one. Click on the{' '}
              <strong>Assignments</strong> tab.
            </Paragraph>
          </Panel>
          <Panel header="What file formats are allowed?" key="assignments-2">
            <Paragraph>
              The platform typically accepts `.zip`, `.pdf`, `.txt`, and code files depending on the
              assignment setup.
            </Paragraph>
          </Panel>
        </Collapse>

        <Divider />

        <Title level={4} id="submissions">
          Submissions
        </Title>
        <Collapse accordion>
          <Panel header="How do I submit my work?" key="submissions-1">
            <Paragraph>
              Open the assignment and upload your solution under the <strong>Submissions</strong>{' '}
              tab.
            </Paragraph>
          </Panel>
          <Panel header="Can I edit or delete a submission?" key="submissions-2">
            <Paragraph>
              You can submit multiple times until the deadline. The most recent submission is used
              for grading.
            </Paragraph>
          </Panel>
        </Collapse>

        <Divider />

        <Title level={4} id="troubleshooting">
          Troubleshooting
        </Title>
        <Collapse accordion>
          <Panel header="Why am I seeing a 403 or 401 error?" key="troubleshooting-1">
            <Paragraph>
              401 means you are not logged in. 403 means you do not have permission to access that
              page.
            </Paragraph>
          </Panel>
          <Panel header="Page not found or broken layout?" key="troubleshooting-2">
            <Paragraph>
              Try logging out and back in. If the issue persists, contact support.
            </Paragraph>
          </Panel>
        </Collapse>

        <Divider />

        <Title level={4} id="contact">
          Need more help?
        </Title>
        <Paragraph>
          Contact us at <a href="mailto:support@example.com">support@example.com</a> and we’ll
          respond as soon as possible.
        </Paragraph>
      </Content>

      <Sider width={220} style={{ background: 'transparent' }} breakpoint="lg" collapsedWidth="0">
        <Anchor
          offsetTop={80}
          items={[
            { key: 'account', href: '#account', title: 'Account & Login' },
            { key: 'assignments', href: '#assignments', title: 'Assignments' },
            { key: 'submissions', href: '#submissions', title: 'Submissions' },
            { key: 'troubleshooting', href: '#troubleshooting', title: 'Troubleshooting' },
            { key: 'contact', href: '#contact', title: 'Contact' },
          ]}
        />
      </Sider>
    </Layout>
  );
};

export default HelpPage;
