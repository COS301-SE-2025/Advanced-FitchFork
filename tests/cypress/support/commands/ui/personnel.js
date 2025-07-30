/**
 * Assigns a user to a module role by searching for their username on the personnel page.
 *
 * @param {Object} params
 * @param {number} params.moduleId - The ID of the module
 * @param {string} params.role - The role to assign (e.g., "Student", "Lecturer", "Tutor")
 * @param {string} params.search - Search username of the user to assign
 */
Cypress.Commands.add("assignUserToModule", ({ moduleId, role, search }) => {
  cy.visit(`/modules/${moduleId}/personnel`);

  cy.get('[data-cy="personnel-role-selector"]')
    .contains(new RegExp(`^${role}$`, 'i'))
    .click();

  cy.get('[data-cy="available-user-search"]').clear().type(search).type("{enter}");

  // ✅ Match any available user row whose data-cy starts with "available-user-row-"
  cy.get('[data-cy="available-user-table"]')
    .find('[data-cy^="available-user-row-"]')
    .first()
    .click();

  cy.get(".ant-transfer-operation button").first().click();

  cy.get('[data-cy="assigned-user-search"]').clear().type(search).type("{enter}");

  cy.get('[data-cy="assigned-user-table"]')
    .find('[data-cy^="assigned-user-row-"]')
    .contains(search)
    .should("exist");
});


/**
 * Unassigns a user from a module role by searching for their username on the personnel page.
 *
 * @param {Object} params
 * @param {number} params.moduleId - The ID of the module
 * @param {string} params.role - The role to unassign from
 * @param {string} params.search - Seach username of the user to unassign
 */
Cypress.Commands.add("unassignUserFromModule", ({ moduleId, role, search }) => {
  cy.visit(`/modules/${moduleId}/personnel`);

  cy.get('[data-cy="personnel-role-selector"]')
    .contains(new RegExp(`^${role}$`, 'i'))
    .click();

  cy.get('[data-cy="assigned-user-search"]').clear().type(search).type("{enter}");

  cy.get('[data-cy="assigned-user-table"]')
    .find('[data-cy^="assigned-user-row-"]')
    .first()
    .click();

  cy.get(".ant-transfer-operation button").last().click();

  cy.get('[data-cy="assigned-user-search"]').clear().type(search).type("{enter}");

  cy.get('[data-cy="assigned-user-table"]')
    .should("not.contain", search);
});

