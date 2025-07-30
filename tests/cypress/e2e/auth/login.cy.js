describe('Login Page', () => {
  beforeEach(() => {
    cy.visit('/login');
  });

  it('renders login form', () => {
    cy.get('[id="username"]').should('exist');
    cy.get('[id="password"]').should('exist');
    cy.get('[type="submit"]').should('exist');
  });

  it('shows validation errors on empty submit', () => {
    cy.get('[type="submit"]').click();
    cy.contains('Please enter your username').should('exist');
    cy.contains('Please enter your password').should('exist');
  });

  it('logs in with valid credentials', () => {
    cy.get('[id="username"]').type('admin');
    cy.get('[id="password"]').type('1');
    cy.get('[type="submit"]').click();

    // adjust this depending on how you redirect after login
    cy.url().should('include', '/dashboard');
  });

  it('does not log in with wrong password', () => {
    cy.get('[id="username"]').type('admin');
    cy.get('[id="password"]').type('wrongpassword');
    cy.get('[type="submit"]').click();

    cy.contains('Invalid student number or password').should('exist');
    cy.url().should('include', '/login');
  });

  it('trims username input', () => {
    cy.get('[id="username"]').type('   admin   ');
    cy.get('[id="password"]').type('1');
    cy.get('[type="submit"]').click();

    cy.url().should('include', '/dashboard');
  });

  it('displays error then clears it on input change', () => {
    cy.get('[type="submit"]').click();
    cy.contains('Please enter your username').should('exist');

    cy.get('[id="username"]').type('admin');
    cy.contains('Please enter your username').should('not.exist');
  });

  it('allows pressing enter to submit form', () => {
    cy.get('[id="username"]').type('admin');
    cy.get('[id="password"]').type('1{enter}');

    cy.url().should('include', '/dashboard');
  });

  it('shows error message from backend response', () => {
    // intercept login request and force a failure
    cy.intercept('POST', '/api/auth/login', {
      statusCode: 401,
      body: { success: false, message: 'Invalid credentials' },
    }).as('login');

    cy.get('[id="username"]').type('admin');
    cy.get('[id="password"]').type('badpass');
    cy.get('[type="submit"]').click();

    cy.wait('@login');
    cy.contains('Invalid credentials').should('exist');
  });
});