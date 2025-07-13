import { Collapse, Typography, Divider } from 'antd';

const { Title, Paragraph, Text } = Typography;
const { Panel } = Collapse;

const HelpAssignments = () => {
  return (
    <div>
      <Title level={3}>Assignments</Title>
      <Paragraph>
        Learn how to access, understand, and manage your assignments within your enrolled modules.
      </Paragraph>

      <Divider />

      <Title level={4}>Assignment Basics</Title>

      <Collapse accordion>
        <Panel header="Where can I find my assignments?" key="1">
          <Paragraph>
            Go to the <Text strong>Modules</Text> section from the sidebar, then select the specific
            module you're enrolled in. From there, click the <Text strong>Assignments</Text> tab to
            view a list of all related assignments.
          </Paragraph>
        </Panel>

        <Panel header="How do I know when an assignment is due?" key="2">
          <Paragraph>
            Each assignment displays a <Text strong>due date</Text> on the list view. Inside the
            assignment page, you&apos;ll also see the full deadline and any grace period details.
          </Paragraph>
        </Panel>

        <Panel header="What does an assignment include?" key="3">
          <Paragraph>An assignment may include:</Paragraph>
          <ul style={{ paddingLeft: '1.5rem' }}>
            <li>
              <Text strong>Specification file:</Text> Describes what&apos;s expected.
            </li>
            <li>
              <Text strong>Starter files or templates:</Text> Provided by your lecturer or tutor.
            </li>
            <li>
              <Text strong>Mark allocation:</Text> Sometimes included as a rubric or memo.
            </li>
          </ul>
        </Panel>

        <Panel header="Can I download all files at once?" key="4">
          <Paragraph>
            Yes. On the assignment page, you will see download buttons or grouped download links for
            available files (specs, templates, etc.).
          </Paragraph>
        </Panel>

        <Panel header="What if I can't see any assignments?" key="5">
          <Paragraph>
            Ensure you are enrolled in the correct module. If you're still having issues, contact
            your lecturer or reach out to{' '}
            <a href="mailto:support@example.com">support@example.com</a>.
          </Paragraph>
        </Panel>
      </Collapse>

      <Divider />

      <Title level={4}>For Staff</Title>
      <Collapse accordion>
        <Panel header="How do I create or upload a new assignment?" key="6">
          <Paragraph>
            Navigate to your module, then click <Text strong>Create Assignment</Text>. Fill in the
            details like title, description, and due date. After creation, you can upload related
            files like the spec, memo, or test files.
          </Paragraph>
        </Panel>

        <Panel header="Can I update an assignment after it's published?" key="7">
          <Paragraph>
            Yes. You can edit details, upload new files, or change the due date until the deadline
            passes. Students will see updates immediately.
          </Paragraph>
        </Panel>
      </Collapse>

      <Divider />

      <Title level={4}>Tips</Title>
      <ul style={{ paddingLeft: '1.5rem' }}>
        <li>
          <Text strong>Check regularly:</Text> New assignments may appear any time during the
          semester.
        </li>
        <li>
          <Text strong>Don&apos;t wait:</Text> Start early to avoid last-minute issues.
        </li>
        <li>
          <Text strong>Understand the spec:</Text> If anything is unclear, ask your lecturer or
          tutor before submitting.
        </li>
      </ul>

      <Divider />

      <Title level={4}>Need Help?</Title>
      <Paragraph>
        Contact your module coordinator, or reach out to{' '}
        <a href="mailto:support@example.com">support@example.com</a> if you're stuck.
      </Paragraph>
    </div>
  );
};

export default HelpAssignments;
