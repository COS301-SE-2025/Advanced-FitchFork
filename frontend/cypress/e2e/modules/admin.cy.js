describe('Admin: Modules Validation', () => {
  let createdCode = '';

  beforeEach(() => {
    cy.loginAs('admin');
    cy.visit('/modules');
  });

  it('should reject invalid module code format', () => {
    cy.intercept('POST', '/api/modules').as('createModule');

    cy.get('[data-cy="control-action-create"]').click();

    cy.get('input[id=code]').type('XYZ12');
    cy.get('input[id=year]').clear().type('2025');
    cy.get('input[id=description]').type('Invalid code test');
    cy.get('input[id=credits]').clear().type('16');
    cy.contains('Create').click();

    cy.wait('@createModule').its('response.statusCode').should('eq', 400);
  });

  it('should accept valid module code format (COS123)', () => {
    cy.intercept('POST', '/api/modules').as('createModule');

    cy.get('[data-cy="control-action-create"]').click();

    const timestamp = Date.now();
    createdCode = `COS${timestamp.toString().slice(-3)}`;

    cy.get('input[id=code]').type(createdCode);
    cy.get('input[id=year]').clear().type(`${new Date().getFullYear()}`);
    cy.get('input[id=description]').type('Valid code test');
    cy.get('input[id=credits]').clear().type('16');
    cy.contains('Create').click();

    cy.wait('@createModule').its('response.statusCode').should('eq', 201);
    cy.contains('Module created').should('exist');
  });

  it('should edit the first visible module', () => {
    cy.intercept('PUT', '/api/modules/*').as('editModule');

    cy.get('[data-cy^="entity-"]').first().within(() => {
      cy.get('[data-cy="entity-action-edit"]').click();
    });

    const newDescription = 'Updated description';
    cy.get('input[id=description]').clear().type(newDescription);
    cy.contains('Save').click();

    cy.wait('@editModule').its('response.statusCode').should('eq', 200);
  });

it('should delete the first visible module', () => {
  cy.intercept('DELETE', '/api/modules/*').as('deleteModule');

  // Step 1: Trigger dropdown hover from inside entity row
  cy.get('[data-cy^="entity-"]').first().within(() => {
    cy.get('[data-cy="entity-action-dropdown"]').trigger('mouseover');
  });

  // Step 2: Click delete from globally rendered dropdown menu
  cy.get('[data-cy="entity-action-delete"]').click();

  // Step 3: Confirm
  cy.contains('Yes').click();

  // Step 4: Assert deletion succeeded
  cy.wait('@deleteModule').its('response.statusCode').should('eq', 200);
  cy.contains('Module deleted').should('exist');
});
});
