/**
 * Creates a new assignment in a module via the UI.
 *
 * @param {Object} params
 * @param {number} params.moduleId - ID of the module
 * @param {string} params.name - Assignment name
 * @param {string} params.description - Assignment description
 * @param {string} params.availableFrom - Format: YYYY-MM-DD HH:mm
 * @param {string} params.dueDate - Format: YYYY-MM-DD HH:mm
 * @param {number} [params.expectedStatus=201] - Expected HTTP status code
 * @returns {Cypress.Chainable<{ assignmentId: number, response: Object }>}
 */
Cypress.Commands.add('createAssignment', ({ moduleId, name, description, availableFrom, dueDate, expectedStatus = 201 }) => {
  cy.visit(`/modules/${moduleId}/assignments`);
  cy.get('[data-cy="view-toggle-table"]').click();

  cy.intercept('POST', `/api/modules/${moduleId}/assignments`).as('createAssignment');

  cy.get('[data-cy="control-action-create"]').click();
  cy.get('[id="name"]').type(name);
  cy.get('[id="description"]').type(description);
  cy.get('[id="available_from"]').clear().type(availableFrom);
  cy.contains('OK').click({ force: true });
  cy.get('[id="due_date"]').clear().type(dueDate);
  cy.contains('OK').click({ force: true });
  cy.get('[data-cy="create-modal-submit"]').click();

  return cy.wait('@createAssignment').then(({ response }) => {
    expect(response.statusCode).to.eq(expectedStatus);
    return {
      assignmentId: response.body?.data?.id,
      response
    };
  });
});

/**
 * Edits an existing assignment via the UI.
 *
 * @param {Object} params
 * @param {number} params.moduleId - Module ID
 * @param {number} params.assignmentId - Assignment ID
 * @param {string} params.search - Search term to locate the assignment in the UI
 * @param {string} params.name - New assignment name
 * @param {string} params.description - New description
 * @param {string} params.availableFrom - Format: YYYY-MM-DD HH:mm
 * @param {string} params.dueDate - Format: YYYY-MM-DD HH:mm
 * @param {number} [params.expectedStatus=200] - Expected HTTP status code
 * @returns {Cypress.Chainable<{ response: Object }>}
 */
Cypress.Commands.add('editAssignment', ({ moduleId, assignmentId, search, name, description, availableFrom, dueDate, expectedStatus = 200 }) => {
  cy.visit(`/modules/${moduleId}/assignments`);
  cy.get('[data-cy="view-toggle-table"]').click();

  cy.intercept('PUT', `/api/modules/${moduleId}/assignments/${assignmentId}`).as('editAssignment');

  cy.get('[data-cy="entity-search"]').clear().type(`${search}{enter}`);

  cy.get(`[data-cy="entity-${assignmentId}"]`).within(() => {
    cy.get('[data-cy="entity-action-dropdown"]').trigger('mouseover');
  });
  cy.get('[data-cy="entity-action-edit"]').click({ force: true });

  cy.get('[id="name"]').clear().type(name);
  cy.get('[id="description"]').clear().type(description);
  cy.get('[id="available_from"]').clear().type(availableFrom);
  cy.contains('OK').click({ force: true });
  cy.get('[id="due_date"]').clear().type(dueDate);
  cy.contains('OK').click({ force: true });

  cy.get('[data-cy="edit-modal-submit"]').click();

  return cy.wait('@editAssignment').then(({ response }) => {
    expect(response.statusCode).to.eq(expectedStatus);
    return { response };
  });
});

/**
 * Deletes an assignment via the UI.
 *
 * @param {Object} params
 * @param {number} params.moduleId - Module ID
 * @param {number} params.assignmentId - Assignment ID
 * @param {string} params.search - Search term to locate the assignment in the table
 * @param {number} [params.expectedStatus=200] - Expected HTTP status code
 * @returns {Cypress.Chainable<{ response: Object }>}
 */
Cypress.Commands.add('deleteAssignment', ({ moduleId, assignmentId, search, expectedStatus = 200 }) => {
  cy.visit(`/modules/${moduleId}/assignments`);
  cy.get('[data-cy="view-toggle-table"]').click();

  cy.intercept('DELETE', `/api/modules/${moduleId}/assignments/${assignmentId}`).as('deleteAssignment');

  // Filter the table so the assignment appears
  cy.get('[data-cy="entity-search"]').clear().type(`${search}{enter}`);

  // Open dropdown and click delete
  cy.get(`[data-cy="entity-${assignmentId}"]`).should('exist').within(() => {
    cy.get('[data-cy="entity-action-dropdown"]').trigger('mouseover');
  });
  cy.get('[data-cy="entity-action-delete"]').click({ force: true });
  cy.contains('Yes').click();

  return cy.wait('@deleteAssignment').then(({ response }) => {
    expect(response.statusCode).to.eq(expectedStatus);
    return { response };
  });
});

/**
 * Performs full assignment setup (config, files, tasks, generate) via the UI.
 *
 * @param {Object} params
 * @param {number} params.moduleId - Module ID
 * @param {number} params.assignmentId - Assignment ID
 */
Cypress.Commands.add('setupAssignment', ({ moduleId, assignmentId }) => {
  expect(moduleId, 'moduleId should be a number').to.be.a('number');
  expect(assignmentId, 'assignmentId should be a number').to.be.a('number');

  cy.visit(`/modules/${moduleId}/assignments`);
  cy.get('[data-cy="view-toggle-table"]').click();

  cy.get('[data-cy="entity-action-setup"]').click();

  // Step 1: Welcome
  cy.contains('Next').click();

  // Step 2: Config
  cy.get('[data-cy="step-config-default"]').click();
  cy.contains('Next').click();

  // Step 3: File Uploads
  cy.contains('Main Files').should('exist');
  cy.get('input[type="file"]').attachFile('java_main.zip');
  cy.wait(100);

  cy.contains('Memo File').should('exist');
  cy.get('input[type="file"]').attachFile('java_memo.zip');
  cy.wait(100);

  cy.contains('Makefile').should('exist');
  cy.get('input[type="file"]').attachFile('java_makefile.zip');
  cy.wait(100);

  cy.contains('Next').click();

  // Step 4: Tasks
  cy.contains('Define Tasks').should('exist');

  for (let i = 1; i <= 3; i++) {
    cy.contains('Add Task').click();
    cy.get('[data-cy="task-card"]').should('have.length.at.least', i);
    cy.get('[data-cy="task-card"]').eq(i - 1).within(() => {
      cy.contains('Edit').click();
      cy.get('input[placeholder="Task Name"]').clear().type(`Task ${i}`);
      cy.get('input[placeholder="Command"]').clear().type(`make task${i}`);
      cy.contains('Save').click();

      cy.contains(`Task ${i}`).should('exist');
      cy.contains(`make task${i}`).should('exist');
    });
  }

  cy.intercept('POST', `/api/modules/${moduleId}/assignments/${assignmentId}/memo_output/generate`).as('generateMemo');
  cy.intercept('POST', `/api/modules/${moduleId}/assignments/${assignmentId}/mark_allocator/generate`).as('generateAllocator');

  cy.contains('Next').should('not.be.disabled').click();

  cy.contains('Generate Memo Output & Mark Allocator', { timeout: 10000 }).should('exist');

  cy.get('[data-cy="generate-memo-mark"]').should('not.be.disabled').click();
  cy.wait('@generateMemo', { timeout: 20000 }).its('response.statusCode').should('eq', 200);
  cy.wait('@generateAllocator', { timeout: 20000 }).its('response.statusCode').should('eq', 200);

  cy.contains('Finish').should('be.visible').and('not.be.disabled').click();
});
