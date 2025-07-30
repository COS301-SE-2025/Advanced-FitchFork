import { getAuthToken } from "@utils/auth";
import { API_BASE_URL } from "@utils/api";

/**
 * Triggers asynchronous generation of memo outputs for a specific assignment.
 * Only accessible by lecturers or admins assigned to the module.
 *
 * @param {object} options
 * @param {number} options.moduleId - Module ID
 * @param {number} options.assignmentId - Assignment ID
 * @returns {Promise<{status: number, body: object}>}
 */
Cypress.Commands.add('apiGenerateMemoOutput', ({ moduleId, assignmentId }) => {
  return cy.window().then((win) => {
    const token = getAuthToken.call(win);
    if (!token) throw new Error("Missing auth token");

    return cy.request({
      method: 'POST',
      url: `${API_BASE_URL}/modules/${moduleId}/assignments/${assignmentId}/memo_output/generate`,
      headers: {
        Authorization: `Bearer ${token}`,
      },
      failOnStatusCode: false, // allow testing validation errors
    }).then((res) =>
      cy.wrap({
        status: res.status,
        body: res.body,
      })
    );
  });
});
