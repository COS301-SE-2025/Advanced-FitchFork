import { getAuthToken, setAuthToken } from "@utils/auth";
import { API_BASE_URL} from "@utils/api";

/**
 * Registers a new user via the API.
 * Returns { status, body } for inspection.
 */
Cypress.Commands.add('apiRegister', ({ username, email, password }) => {
  return cy.request({
    method: 'POST',
    url: `${API_BASE_URL}/auth/register`,
    body: { username, email, password },
    failOnStatusCode: false,
  }).then((res) =>
    cy.wrap({
      status: res.status,
      body: res.body,
    })
  );
});

/**
 * Logs in an existing user via the API.
 * Sets token in localStorage on success.
 * Returns { status, body } for inspection.
 */
Cypress.Commands.add('apiLogin', ({ username, password }) => {
  return cy.request({
    method: 'POST',
    url: `${API_BASE_URL}/auth/login`,
    body: { username, password },
    failOnStatusCode: false,
  }).then((res) => {
    const { status, body } = res;

    if (status === 200 && body?.data?.token) {
      return cy.window().then((win) => {
        setAuthToken.call(win, body.data);
        return res;
      });
    }

    // Return response even if failed, still chainable
    return res;
  });
});

/**
 * Logs in a user by role using the API and stores the token using setAuthToken.
 *
 * @example
 *   cy.apiLoginAs('admin')   // logs in using admin credentials via API
 *
 * @param {string} role - One of: 'admin', 'lecturer', 'assistant_lecturer', 'tutor', 'student'
 */
Cypress.Commands.add('apiLoginAs', (role) => {
  return cy.fixture('users').then((users) => {
    const { username, password } = users[role];

    expect(username, 'username must exist').to.be.a('string');
    expect(password, 'password must exist').to.be.a('string');

    // return the Cypress chain
    return cy.apiLogin({ username, password }).then((res) => {
      expect(res.status).to.eq(200);
      expect(res.body?.data?.token, 'token must exist in response').to.be.a('string');
    });
  });
});

/**
 * Checks if the current user has a specific role in a module.
 * Requires prior login (token from localStorage).
 */
Cypress.Commands.add('apiHasRole', ({ moduleId, role }) => {
  return cy.window().then((win) => {
    const token = getAuthToken.call(win);
    if (!token) throw new Error('Missing auth token');

    return cy.request({
      method: 'GET',
      url: `/api/auth/has-role?module_id=${moduleId}&role=${role}`,
      headers: {
        Authorization: `Bearer ${token}`,
      },
      failOnStatusCode: false,
    }).then((res) =>
      cy.wrap({
        status: res.status,
        body: res.body,
      })
    );
  });
});

/**
 * Fetches the user's role in a module (if any).
 * Requires prior login (token from localStorage).
 */
Cypress.Commands.add('apiGetModuleRole', ({ moduleId }) => {
  return cy.window().then((win) => {
    const token = getAuthToken.call(win);
    if (!token) throw new Error('Missing auth token');

    return cy.request({
      method: 'GET',
      url: `${API_BASE_URL}/auth/module-role?module_id=${moduleId}`,
      headers: {
        Authorization: `Bearer ${token}`,
      },
      failOnStatusCode: false,
    }).then((res) =>
      cy.wrap({
        status: res.status,
        body: res.body,
      })
    );
  });
});


/**
 * Gets the currently authenticated user (requires a valid token).
 *
 * @returns {Promise<{status: number, body: object}>}
 */
Cypress.Commands.add('apiGetCurrentUser', () => {
  return cy.window().then((win) => {
    const token = getAuthToken.call(win);
    if (!token) throw new Error("Missing auth token");

    return cy.request({
      method: 'GET',
      url: `${API_BASE_URL}/auth/me`,
      headers: {
        Authorization: `Bearer ${token}`,
      },
      failOnStatusCode: false,
    }).then((res) =>
      cy.wrap({
        status: res.status,
        body: res.body,
      })
    );
  });
});