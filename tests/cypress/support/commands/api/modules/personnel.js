import { getAuthToken } from "@utils/auth";
import { API_BASE_URL } from "@utils/api";

/**
 * Assigns or updates users to a module with a specific role (admin or lecturer access required).
 * @param {number} moduleId - ID of the module
 * @param {Array<number>} userIds - List of user IDs
 * @param {string} role - One of: 'student', 'tutor', 'assistant_lecturer', 'lecturer'
 * @returns {Promise<{status: number, body: object}>}
 */
Cypress.Commands.add('apiAssignPersonnel', ({ moduleId, userIds, role }) => {
  return cy.window().then((win) => {
    const token = getAuthToken.call(win);
    if (!token) throw new Error("Missing auth token");

    return cy.request({
      method: 'POST',
      url: `${API_BASE_URL}/modules/${moduleId}/personnel`,
      headers: {
        Authorization: `Bearer ${token}`,
      },
      body: {
        user_ids: userIds,
        role,
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
 * Removes users from a specific role in a module (admin or lecturer access required).
 * @param {number} moduleId - ID of the module
 * @param {Array<number>} userIds - List of user IDs to remove
 * @param {string} role - One of: 'student', 'tutor', 'assistant_lecturer', 'lecturer'
 * @returns {Promise<{status: number, body: object}>}
 */
Cypress.Commands.add('apiRemovePersonnel', ({ moduleId, userIds, role }) => {
  return cy.window().then((win) => {
    const token = getAuthToken.call(win);
    if (!token) throw new Error("Missing auth token");

    return cy.request({
      method: 'DELETE',
      url: `${API_BASE_URL}/modules/${moduleId}/personnel`,
      headers: {
        Authorization: `Bearer ${token}`,
      },
      body: {
        user_ids: userIds,
        role,
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
 * Fetches eligible users for a module (i.e. users with no role in that module).
 * Supports optional query params: query, email, username, page, per_page, sort.
 *
 * @param {number} moduleId - ID of the module
 * @param {object} [params] - Optional query params
 * @returns {Promise<{status: number, body: object}>}
 */
Cypress.Commands.add('apiGetEligibleUsers', ({ moduleId, params = {} }) => {
  const query = new URLSearchParams(params).toString();

  return cy.window().then((win) => {
    const token = getAuthToken.call(win);
    if (!token) throw new Error("Missing auth token");

    return cy.request({
      method: 'GET',
      url: `${API_BASE_URL}/modules/${moduleId}/personnel/eligible${query ? `?${query}` : ''}`,
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
