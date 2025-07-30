import "./personnel";
import "./assignments";

import { getAuthToken } from "@utils/auth";
import { API_BASE_URL } from "@utils/api";

/**
 * Creates a new module via the API (admin-only).
 * Returns { status, body } for inspection.
 */
Cypress.Commands.add('apiCreateModule', ({ code, year, description, credits }) => {
  return cy.window().then((win) => {
    const token = getAuthToken.call(win);
    if (!token) throw new Error("Missing auth token");

    return cy.request({
      method: 'POST',
      url: `${API_BASE_URL}/modules`,
      headers: {
        Authorization: `Bearer ${token}`,
      },
      body: {
        code,
        year,
        description,
        credits,
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
 * Updates a module via the API (admin-only).
 * Requires full body: code, year, description, credits.
 * Returns { status, body } for inspection.
 */
Cypress.Commands.add('apiUpdateModule', ({ moduleId, code, year, description, credits }) => {
  return cy.window().then((win) => {
    const token = getAuthToken.call(win);
    if (!token) throw new Error("Missing auth token");

    return cy.request({
      method: 'PUT',
      url: `${API_BASE_URL}/modules/${moduleId}`,
      headers: {
        Authorization: `Bearer ${token}`,
      },
      body: {
        code,
        year,
        description,
        credits,
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
 * Deletes a module by ID via the API (admin-only).
 * Returns { status, body } for inspection.
 */
Cypress.Commands.add('apiDeleteModule', ({moduleId}) => {
  return cy.window().then((win) => {
    const token = getAuthToken.call(win);
    if (!token) throw new Error("Missing auth token");

    return cy.request({
      method: 'DELETE',
      url: `${API_BASE_URL}/modules/${moduleId}`,
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
