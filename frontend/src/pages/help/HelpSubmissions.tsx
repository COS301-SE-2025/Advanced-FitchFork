import { Collapse, Typography, Divider } from 'antd';

const { Title, Paragraph, Text } = Typography;
const { Panel } = Collapse;

const HelpSubmissions = () => {
  return (
    <div>
      <Title level={3}>Submissions</Title>
      <Paragraph>
        Learn how to upload your assignment submissions, track submission history, and view feedback
        from staff.
      </Paragraph>

      <Divider />

      <Title level={4}>For Students</Title>
      <Collapse accordion>
        <Panel header="How do I submit an assignment?" key="1">
          <Paragraph>
            Open the relevant <Text strong>module</Text>, then go to the{' '}
            <Text strong>Assignments</Text> tab. Click on the assignment, then select the{' '}
            <Text strong>Submissions</Text> section.
          </Paragraph>
          <Paragraph>
            Use the <Text strong>Upload</Text> button to select and submit your file(s). Accepted
            formats depend on the assignment configuration.
          </Paragraph>
        </Panel>

        <Panel header="Can I submit more than once?" key="2">
          <Paragraph>
            Yes. You may upload multiple times before the due date. Only your{' '}
            <Text strong>latest submission</Text> will be marked, unless stated otherwise.
          </Paragraph>
        </Panel>

        <Panel header="How do I know if my submission was successful?" key="3">
          <Paragraph>
            After uploading, you will see your submission listed under the{' '}
            <Text strong>Submissions</Text> table along with a timestamp. A success message is also
            shown.
          </Paragraph>
        </Panel>

        <Panel header="Where do I see my grade and feedback?" key="4">
          <Paragraph>
            After grading, open the assignment and go to <Text strong>Submissions</Text>. Click on
            your submission to view:
          </Paragraph>
          <ul style={{ paddingLeft: '1.5rem' }}>
            <li>
              <Text strong>Mark breakdown</Text>
            </li>
            <li>
              <Text strong>Feedback</Text> comments
            </li>
            <li>
              <Text strong>Uploaded result files</Text> (if applicable)
            </li>
          </ul>
        </Panel>
      </Collapse>

      <Divider />

      <Title level={4}>For Staff</Title>
      <Collapse accordion>
        <Panel header="How do I view student submissions?" key="5">
          <Paragraph>
            Navigate to your module&apos;s <Text strong>Assignments</Text> tab, select the
            assignment, then open the <Text strong>Submissions</Text> section. You can view all
            student submissions, download them, and access details.
          </Paragraph>
        </Panel>

        <Panel header="Can I grade submissions manually?" key="6">
          <Paragraph>
            Yes. Depending on the setup, you may provide scores and feedback manually or review
            results from automated tests before publishing.
          </Paragraph>
        </Panel>

        <Panel header="How is feedback delivered to students?" key="7">
          <Paragraph>
            Once grading is complete, students can open their submission to view detailed feedback,
            scores, and attached output files.
          </Paragraph>
        </Panel>
      </Collapse>

      <Divider />

      <Title level={4}>Tips</Title>
      <ul style={{ paddingLeft: '1.5rem' }}>
        <li>
          <Text strong>Don&apos;t wait until the last minute:</Text> Upload early to avoid technical
          issues.
        </li>
        <li>
          <Text strong>Read the spec carefully:</Text> Some assignments have strict format or
          filename requirements.
        </li>
        <li>
          <Text strong>Check feedback regularly:</Text> It helps improve your future work.
        </li>
      </ul>

      <Divider />

      <Title level={4}>Need Help?</Title>
      <Paragraph>
        If you're experiencing issues with submission uploads or feedback visibility, please contact{' '}
        <a href="mailto:support@example.com">support@example.com</a>.
      </Paragraph>
    </div>
  );
};

export default HelpSubmissions;
