describe("Admin: Module CRUD Operations", () => {
  beforeEach(() => {
    // Start a new session as admin and navigate to modules page
    cy.session('admin', () => {
      cy.loginAs('admin');
    });
    cy.visit('/modules');
  });

  it("can create module", () => {
    cy.intercept('POST', '/api/modules').as('createModule');

    // STEP 1: Click the "Create Module" button
    cy.get('[data-cy="control-action-create"]').click();

    // STEP 2: Fill in the form with a unique module code
    const timestamp = Date.now();
    const createdCode = `COS${timestamp.toString().slice(-3)}`;

    cy.get('input[id=code]').type(createdCode);
    cy.get('input[id=year]').clear().type(`${new Date().getFullYear()}`);
    cy.get('input[id=description]').type('Valid code test');
    cy.get('input[id=credits]').clear().type('16');

    // STEP 3: Submit the form
    cy.contains('Create').click();

    cy.wait('@createModule').its('response.statusCode').should('eq', 201);
  });

  it("can edit module", () => {
    cy.intercept('PUT', '/api/modules/*').as('editModule');

    // STEP 1: Open the edit form for the first module in the list
    cy.get('[data-cy^="entity-"]').first().within(() => {
      cy.get('[data-cy="entity-action-edit"]').click();
    });

    // STEP 2: Change the description field
    const newDescription = 'Updated description';
    cy.get('input[id=description]').clear().type(newDescription);

    // STEP 3: Submit the form
    cy.contains('Save').click();

    cy.wait('@editModule').its('response.statusCode').should('eq', 200);
  });

  it("can delete module", () => {
    cy.intercept('DELETE', '/api/modules/*').as('deleteModule');

    // STEP 1: Hover over the module row to reveal action dropdown
    cy.get('[data-cy^="entity-"]').first().within(() => {
      cy.get('[data-cy="entity-action-dropdown"]').trigger('mouseover');
    });

    // STEP 2: Click the "Delete" option in the dropdown menu
    cy.get('[data-cy="entity-action-delete"]').click();

    // STEP 3: Confirm the deletion in the modal
    cy.contains('Yes').click();

    // STEP 4: Assert that the deletion succeeded via message
    cy.wait('@deleteModule').its('response.statusCode').should('eq', 200);
    cy.contains('Module deleted').should('exist');
  });

  it("should reject invalid module code format", () => {
    cy.intercept('POST', '/api/modules').as('createModule');

    // STEP 1: Click the "Create Module" button
    cy.get('[data-cy="control-action-create"]').click();

    // STEP 2: Fill in the form with an invalid module code
    cy.get('input[id=code]').type('XYZ12'); // Invalid: only 2 digits

    cy.get('input[id=year]').clear().type(`${new Date().getFullYear()}`);
    cy.get('input[id=description]').type('Invalid code test');
    cy.get('input[id=credits]').clear().type('16');

    // STEP 3: Submit the form
    cy.contains('Create').click();

    // STEP 4: Wait for the request and assert it failed validation
    cy.wait('@createModule').its('response.statusCode').should('eq', 400);
  });
});
