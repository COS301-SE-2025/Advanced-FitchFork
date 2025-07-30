/**
 * Logs in a user using a role defined in fixtures/users.json.
 *
 * @example
 *   cy.loginAs('admin')      // logs in using admin credentials
 *   cy.loginAs('student')    // logs in using student credentials
 *
 * @param {string} role - One of: 'admin', 'lecturer', 'assistant_lecturer', 'tutor', 'student'
 */
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

/**
 * Registers a new user via the UI.
 * Use this in tests that validate signup flows or when you want to dynamically generate test users.
 *
 * @example
 *   cy.register({
 *     username: 'u10000001',
 *     email: 'test1@example.com',
 *     password: 'pass1234'
 *   })
 *
 * @param {Object} params
 * @param {string} params.username - Must follow the required format (e.g. u00000001)
 * @param {string} params.email - A valid email address
 * @param {string} [params.password='pass1234'] - Password to use (must match confirmation)
 */
Cypress.Commands.add('register', ({ username, email, password = 'pass1234' }) => {
  cy.visit('/signup');

  cy.get('input[id="username"]').type(username);
  cy.get('input[id="email"]').type(email);
  cy.get('input[id="confirmPassword"]').first().type(password);
  cy.get('input[id="confirmPassword"]').last().type(password);
  cy.get('[type="submit"]').click();

  cy.url().should('include', '/login');
});