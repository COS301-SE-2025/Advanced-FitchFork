describe("Admin: User Operations", () => {
  const timestamp = Date.now();
  const username = `testuser_${timestamp}`;
  const email = `${username}@test.com`;
  const password = 'changeme123';

  let userId = null;

  beforeEach(() => {
    cy.session('admin', () => {
      cy.apiLoginAs('admin');
    });
  });

  it("can create a user", () => {
    cy.createUser({ username, email, password }).then(({ userId: id }) => {
      userId = id;
      cy.get('[data-cy="entity-search"]').clear().type(`${username}{enter}`);
      cy.get(`[data-cy="entity-${userId}"]`).should('exist');
    });
  });

  it("can delete the user", () => {
    cy.visit("/users");
    cy.get('[data-cy="entity-search"]').clear().type(`${username}{enter}`);

    cy.deleteUser({ userId, search: username });

    // Re-search to verify the user is gone
    cy.get('[data-cy="entity-search"]').clear().type(`${username}{enter}`);

    // Expect no matching entity
    cy.get(`[data-cy="entity-${userId}"]`).should('not.exist');
  });

});
