Cypress.Commands.add('createUser', ({ username, email, password = 'changeme123', expectedStatus = 201 }) => {
  cy.visit('/users');
  cy.get('[data-cy="view-toggle-table"]').click();

  cy.intercept('POST', '/api/users').as('createUser');

  cy.get('[data-cy="control-action-create"]').click();
  cy.get('input[id="username"]').type(username);
  cy.get('input[id="email"]').type(email);
  cy.get('input[id="password"]').clear().type(password);
  cy.contains('Create').click();

  return cy.wait('@createUser').then(({ response }) => {
    expect(response.statusCode).to.eq(expectedStatus);
    return {
      userId: response.body?.data?.id,
      response,
    };
  });
});

Cypress.Commands.add('deleteUser', ({ userId, search, expectedStatus = 200 }) => {
  cy.visit('/users');
  cy.get('[data-cy="view-toggle-table"]').click();

  cy.intercept('DELETE', `/api/users/${userId}`).as('deleteUser');

  cy.get('[data-cy="entity-search"]').clear().type(`${search}{enter}`);
  cy.get(`[data-cy="entity-${userId}"]`).within(() => {
    cy.get('[data-cy="entity-action-dropdown"]').trigger('mouseover');
  });

  cy.get('[data-cy="entity-action-delete"]').click();
  cy.contains('Yes').click();

  return cy.wait('@deleteUser').then(({ response }) => {
    expect(response.statusCode).to.eq(expectedStatus);
    return { response };
  });
});
