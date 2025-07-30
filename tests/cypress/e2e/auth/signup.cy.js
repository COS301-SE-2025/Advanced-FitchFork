describe('Signup Page', () => {
  beforeEach(() => {
    cy.visit('/signup');
  });

  it('renders signup form', () => {
    cy.get('input[placeholder="u00000000"]').should('exist');
    cy.get('input[placeholder="student@up.ac.za"]').should('exist');
    cy.get('input[placeholder="••••••••"]').should('have.length', 2); // password + confirm
    cy.get('[type="submit"]').contains('Create Account').should('exist');
  });

  it('shows validation errors on empty submit', () => {
    cy.get('[type="submit"]').click();
    cy.contains('Please enter your username').should('exist');
    cy.contains('Please enter your email').should('exist');
    cy.contains('Please enter your password').should('exist');
    cy.contains('Please confirm your password').should('exist');
  });

  it('shows error on mismatched passwords', () => {
    cy.get('input[placeholder="u00000000"]').type('testuser123');
    cy.get('input[placeholder="student@up.ac.za"]').type('user@example.com');
    cy.get('input[placeholder="••••••••"]').first().type('password1');
    cy.get('input[placeholder="••••••••"]').last().type('password2');
    cy.get('[type="submit"]').click();

    cy.contains('Passwords do not match').should('exist');
  });

  it('shows password validation errors', () => {
    cy.get('input[placeholder="u00000000"]').type('testuser456');
    cy.get('input[placeholder="student@up.ac.za"]').type('valid@email.com');
    cy.get('input[placeholder="••••••••"]').first().type('short');
    cy.get('input[placeholder="••••••••"]').last().type('short');
    cy.get('[type="submit"]').click();

    cy.contains('Password must be at least 8 characters long').should('exist');

    cy.get('input[placeholder="••••••••"]').first().clear().type('allletters');
    cy.get('input[placeholder="••••••••"]').last().clear().type('allletters');
    cy.get('[type="submit"]').click();

    cy.contains('Password must include at least one letter and one number').should('exist');
  });

  it('submits form and redirects on success', () => {
    const suffix = Math.floor(Math.random() * 100000);
    cy.get('input[placeholder="u00000000"]').type(`newuser${suffix}`);
    cy.get('input[placeholder="student@up.ac.za"]').type(`newuser${suffix}@test.com`);
    cy.get('input[placeholder="••••••••"]').first().type('pass1234');
    cy.get('input[placeholder="••••••••"]').last().type('pass1234');
    cy.get('[type="submit"]').click();

    cy.url().should('include', '/login');
  });

  it('shows backend error (e.g. username/email exists)', () => {
    cy.intercept('POST', '/api/auth/register', {
      statusCode: 400,
      body: { success: false, message: 'Username already taken' },
    }).as('signup');

    cy.get('input[placeholder="u00000000"]').type('existinguser');
    cy.get('input[placeholder="student@up.ac.za"]').type('existing@example.com');
    cy.get('input[placeholder="••••••••"]').first().type('pass1234');
    cy.get('input[placeholder="••••••••"]').last().type('pass1234');
    cy.get('[type="submit"]').click();

    cy.wait('@signup');
    cy.contains('Username already taken').should('exist');
  });
});
