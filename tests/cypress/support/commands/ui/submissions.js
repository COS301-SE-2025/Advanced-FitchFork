/**
 * Submits an assignment via the UI using the "Submit Assignment" modal.
 *
 * @example
 *   cy.submitAssignment({
 *     moduleId: 1,
 *     assignmentId: 2,
 *     fixtureFile: 'java_submission.zip',
 *     isPractice: true,
 *     expectedStatus: 200,
 *   }).then(({ submissionId }) => {
 *     cy.log(`Submission ID: ${submissionId}`);
 *   });
 *
 * @param {Object} params
 * @param {number} params.moduleId - Module ID
 * @param {number} params.assignmentId - Assignment ID
 * @param {string} [params.fixtureFile='java_submission.zip'] - Fixture file to upload
 * @param {boolean} [params.isPractice=false] - Whether it's a practice submission
 * @param {number} [params.expectedStatus=200] - Expected HTTP status code
 * @returns {Cypress.Chainable<{ submissionId?: number, response: Object }>}
 */
Cypress.Commands.add('submitAssignment', ({
  moduleId,
  assignmentId,
  fixtureFile = 'java_submission.zip',
  isPractice = false,
  expectedStatus = 200,
}) => {
  expect(moduleId, 'moduleId should be a number').to.be.a('number');
  expect(assignmentId, 'assignmentId should be a number').to.be.a('number');

  cy.intercept('POST', `/api/modules/${moduleId}/assignments/${assignmentId}/submissions`).as('submitAssignment');

  cy.visit(`/modules/${moduleId}/assignments/${assignmentId}`);
  cy.get('[data-cy="view-toggle-table"]').click();
  cy.contains('Submit Assignment').should('be.visible').click();

  cy.get('input[type="file"]').should('exist').attachFile(fixtureFile);
  cy.wait(500); // wait for rendering

  if (isPractice) {
    cy.get('label').contains('This is a practice submission').click();
  }

  cy.get('[data-cy="submit-modal-submit"]').should('not.be.disabled').click();

  return cy.wait('@submitAssignment').then(({ response }) => {
    expect(response.statusCode).to.eq(expectedStatus);

    return {
      submissionId: response.body?.data?.id,
      response,
    };
  });
});
