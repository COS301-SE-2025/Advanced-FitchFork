import { getAuthToken } from "@utils/auth";
import { API_BASE_URL } from "@utils/api";

/**
 * Creates a new task for an assignment (admin or lecturer access required).
 *
 * @param {object} options
 * @param {number} options.moduleId - ID of the module
 * @param {number} options.assignmentId - ID of the assignment
 * @param {number} options.task_number - Task number (must be unique within assignment)
 * @param {string} options.command - Shell command to execute
 * @returns {Promise<{status: number, body: object}>}
 */
Cypress.Commands.add('apiCreateAssignmentTask', ({
  moduleId,
  assignmentId,
  task_number,
  command,
}) => {
  return cy.window().then((win) => {
    const token = getAuthToken.call(win);
    if (!token) throw new Error("Missing auth token");

    return cy.request({
      method: 'POST',
      url: `${API_BASE_URL}/modules/${moduleId}/assignments/${assignmentId}/tasks`,
      headers: {
        Authorization: `Bearer ${token}`,
      },
      body: {
        task_number,
        command,
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
 * Updates a task's command and name in an assignment.
 * Only accessible by lecturers/admins assigned to the module.
 *
 * @param {object} options
 * @param {number} options.moduleId - Module ID
 * @param {number} options.assignmentId - Assignment ID
 * @param {number} options.taskId - Task ID
 * @param {string} options.command - New shell command to execute
 * @param {string} options.name - New name for the task
 * @returns {Promise<{status: number, body: object}>}
 */
Cypress.Commands.add('apiUpdateAssignmentTask', ({
  moduleId,
  assignmentId,
  taskId,
  command,
  name,
}) => {
  return cy.window().then((win) => {
    const token = getAuthToken.call(win);
    if (!token) throw new Error('Missing auth token');

    return cy.request({
      method: 'PUT',
      url: `${API_BASE_URL}/modules/${moduleId}/assignments/${assignmentId}/tasks/${taskId}`,
      headers: {
        Authorization: `Bearer ${token}`,
      },
      body: {
        command,
        name,
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
 * Deletes a task from an assignment (admin or lecturer access required).
 *
 * @param {object} options
 * @param {number} options.moduleId - Module ID
 * @param {number} options.assignmentId - Assignment ID
 * @param {number} options.taskId - Task ID
 * @returns {Promise<{status: number, body: object}>}
 */
Cypress.Commands.add('apiDeleteAssignmentTask', ({
  moduleId,
  assignmentId,
  taskId,
}) => {
  return cy.window().then((win) => {
    const token = getAuthToken.call(win);
    if (!token) throw new Error('Missing auth token');

    return cy.request({
      method: 'DELETE',
      url: `${API_BASE_URL}/modules/${moduleId}/assignments/${assignmentId}/tasks/${taskId}`,
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
