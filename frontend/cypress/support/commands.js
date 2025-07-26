Cypress.Commands.add('loginAs', (role) => {
  cy.fixture('users').then((users) => {
    const { username, password } = users[role];

    cy.visit('/login');
    cy.get('input[id=username]').type(username);
    cy.get('input[id=password]').type(password);
    cy.get('button[type=submit]').click();

    cy.url().should('include', '/dashboard');
  });
});

export {};