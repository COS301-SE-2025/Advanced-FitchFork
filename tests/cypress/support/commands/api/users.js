import { getAuthToken } from "@utils/auth";
import { API_BASE_URL } from "@utils/api";

/**
 * Creates a single non-admin user via the API (admin-only).
 * Returns { status, body } for inspection.
 */
Cypress.Commands.add('apiCreateUser', ({ username, email, password }) => {
  return cy.window().then((win) => {
    const token = getAuthToken.call(win);
    if (!token) throw new Error("Missing auth token");

    return cy.request({
      method: 'POST',
      url: `${API_BASE_URL}/users`,
      headers: {
        Authorization: `Bearer ${token}`,
      },
      body: {
        username,
        email,
        password,
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
 * Creates multiple non-admin users via the API (admin-only).
 * Expects an array of users.
 * Returns { status, body } for inspection.
 */
Cypress.Commands.add('apiCreateUsersBulk', (users) => {
  return cy.window().then((win) => {
    const token = getAuthToken.call(win);
    if (!token) throw new Error("Missing auth token");

    return cy.request({
      method: 'POST',
      url: `${API_BASE_URL}/users/bulk`,
      headers: {
        Authorization: `Bearer ${token}`,
      },
      body: {
        users,
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
 * Updates a non-admin user's information via the API (admin-only).
 * Accepts any subset of fields: { username, email, admin }.
 * Returns { status, body } for inspection.
 */
Cypress.Commands.add('apiUpdateUser', ({ userId, username, email, admin }) => {
  return cy.window().then((win) => {
    const token = getAuthToken.call(win);
    if (!token) throw new Error("Missing auth token");

    const body = {};
    if (username !== undefined) body.username = username;
    if (email !== undefined) body.email = email;
    if (admin !== undefined) body.admin = admin;

    return cy.request({
      method: 'PUT',
      url: `${API_BASE_URL}/users/${userId}`,
      headers: {
        Authorization: `Bearer ${token}`,
      },
      body,
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
 * Deletes a user by ID via the API (admin-only).
 * Users cannot delete themselves.
 * Returns { status, body } for inspection.
 */
Cypress.Commands.add('apiDeleteUser', (userId) => {
  return cy.window().then((win) => {
    const token = getAuthToken.call(win);
    if (!token) throw new Error("Missing auth token");

    return cy.request({
      method: 'DELETE',
      url: `${API_BASE_URL}/users/${userId}`,
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
