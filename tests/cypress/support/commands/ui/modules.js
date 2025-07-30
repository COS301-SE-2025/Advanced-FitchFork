/**
 * Creates a new module using the admin UI flow.
 * Assumes the user is already logged in and authorized.
 *
 * @example
 *   cy.createModule({
 *     code: 'MOD123',
 *     year: 2025,
 *     description: 'Test module',
 *     credits: 12,
 *     expectedStatus: 201,
 *   }).then(({ moduleId }) => {
 *     cy.get(`[data-cy="entity-${moduleId}"]`).should('exist');
 *   });
 *
 * @param {Object} params
 * @param {string} params.code - Unique module code (e.g. 'MOD123')
 * @param {number} params.year - Academic year (e.g. 2025)
 * @param {string} params.description - Module description
 * @param {number} [params.credits=16] - Number of credits
 * @param {number} [params.expectedStatus=201] - Expected HTTP status code (default: 201)
 * @returns {Cypress.Chainable<{ moduleId?: number, response: Object }>}
 */
Cypress.Commands.add('createModule', ({ code, year, description, credits = 16, expectedStatus = 201 }) => {
  cy.visit('/modules');
  cy.get('[data-cy="view-toggle-table"]').click();

  cy.intercept('POST', '/api/modules').as('createModule');

  cy.get('[data-cy="control-action-create"]').click();
  cy.get('input[id=code]').type(code);
  cy.get('input[id=year]').clear().type(`${year}`);
  cy.get('input[id=description]').type(description);
  cy.get('input[id=credits]').clear().type(`${credits}`);
  cy.contains('Create').click();

  return cy.wait('@createModule').then(({ response }) => {
    expect(response.statusCode).to.eq(expectedStatus);
    return {
      moduleId: response.body?.data?.id,
      response,
    };
  });
});

/**
 * Edits an existing module using the admin UI flow.
 * Assumes the user is already logged in and authorized.
 *
 * @example
 *   cy.editModule({
 *     moduleId: 101,
 *     search: 'MOD101',
 *     code: 'MOD999',
 *     year: 2026,
 *     description: 'Updated module',
 *     credits: 24,
 *     expectedStatus: 200,
 *   });
 *
 * @param {Object} params
 * @param {number} params.moduleId - ID of the module to edit
 * @param {string} params.search - Text to locate module in UI (typically current code)
 * @param {string} params.code - New module code
 * @param {number} params.year - New academic year
 * @param {string} params.description - New module description
 * @param {number} params.credits - Updated number of credits
 * @param {number} [params.expectedStatus=200] - Expected HTTP status code
 * @returns {Cypress.Chainable<{ response: Object }>}
 */
Cypress.Commands.add('editModule', ({ moduleId, search, code, year, description, credits, expectedStatus = 200 }) => {
  cy.visit('/modules');
  cy.get('[data-cy="view-toggle-table"]').click();

  cy.intercept('PUT', `/api/modules/${moduleId}`).as('editModule');

  cy.get('[data-cy="entity-search"]').clear().type(`${search}{enter}`);
  cy.get(`[data-cy="entity-${moduleId}"]`).within(() => {
    cy.get('[data-cy="entity-action-edit"]').click();
  });

  cy.get('input[id=code]').clear().type(code);
  cy.get('input[id=year]').clear().type(`${year}`);
  cy.get('input[id=description]').clear().type(description);
  cy.get('input[id=credits]').clear().type(`${credits}`);
  cy.contains('Save').click();

  return cy.wait('@editModule').then(({ response }) => {
    expect(response.statusCode).to.eq(expectedStatus);
    return { response };
  });
});


/**
 * Deletes a module using the admin UI flow.
 * Assumes the user is already logged in and authorized.
 *
 * @example
 *   cy.deleteModule({
 *     moduleId: 101,
 *     search: 'MOD999',
 *     expectedStatus: 200,
 *   });
 *
 * @param {Object} params
 * @param {number} params.moduleId - ID of the module to delete
 * @param {string} params.search - Text to locate the module in the UI (usually module code)
 * @param {number} [params.expectedStatus=200] - Expected HTTP status code for deletion
 * @returns {Cypress.Chainable<{ response: Object }>}
 */
Cypress.Commands.add('deleteModule', ({ moduleId, search, expectedStatus = 200 }) => {
  cy.visit('/modules');
  cy.get('[data-cy="view-toggle-table"]').click();

  cy.intercept('DELETE', `/api/modules/${moduleId}`).as('deleteModule');

  cy.get('[data-cy="entity-search"]').clear().type(`${search}{enter}`);
  cy.get(`[data-cy="entity-${moduleId}"]`).within(() => {
    cy.get('[data-cy="entity-action-dropdown"]').trigger('mouseover');
  });

  cy.get('[data-cy="entity-action-delete"]').click();
  cy.contains('Yes').click();

  return cy.wait('@deleteModule').then(({ response }) => {
    expect(response.statusCode).to.eq(expectedStatus);
    return { response };
  });
});

