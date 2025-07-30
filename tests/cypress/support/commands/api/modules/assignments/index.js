import "./tasks";
import "./files";
import "./memo_output";
import "./mark_allocator";
import "./submissions";

import { getAuthToken } from "@utils/auth";
import { API_BASE_URL } from "@utils/api";

/**
 * Creates a new assignment in a module (admin or lecturer access required).
 *
 * @param {object} options
 * @param {number} options.moduleId - Module ID (required)
 * @param {string} options.name - Assignment name (required)
 * @param {string} [options.assignment_type='assignment'] - Must be 'assignment' or 'practical'
 * @param {string} options.available_from - ISO8601 datetime string (required)
 * @param {string} options.due_date - ISO8601 datetime string (required)
 * @param {string} [options.description] - Optional assignment description
 *
 * @returns {Promise<{status: number, body: object}>}
 */
Cypress.Commands.add('apiCreateAssignment', ({
  moduleId,
  name,
  assignment_type = 'assignment',
  available_from,
  due_date,
  description,
}) => {
  if (!moduleId || !name || !available_from || !due_date) {
    throw new Error("Missing required assignment fields");
  }

  return cy.window().then((win) => {
    const token = getAuthToken.call(win);
    if (!token) throw new Error("Missing auth token");

    return cy.request({
      method: 'POST',
      url: `${API_BASE_URL}/modules/${moduleId}/assignments`,
      headers: {
        Authorization: `Bearer ${token}`,
      },
      body: {
        name,
        assignment_type,
        available_from,
        due_date,
        ...(description && { description }),
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
 * Updates a single assignment in a module (admin or lecturer access).
 *
 * @param {object} options
 * @param {number} options.moduleId - Module ID
 * @param {number} options.assignmentId - Assignment ID
 * @param {string} options.name - Name of the assignment
 * @param {string} options.assignment_type - "assignment" or "practical"
 * @param {string} options.available_from - ISO 8601 datetime
 * @param {string} options.due_date - ISO 8601 datetime
 * @param {string} [options.description] - Optional
 */
Cypress.Commands.add('apiUpdateAssignment', ({
  moduleId,
  assignmentId,
  name,
  assignment_type,
  available_from,
  due_date,
  description,
}) => {
  return cy.window().then((win) => {
    const token = getAuthToken.call(win);
    if (!token) throw new Error("Missing auth token");

    return cy.request({
      method: 'PUT',
      url: `${API_BASE_URL}/modules/${moduleId}/assignments/${assignmentId}`,
      headers: {
        Authorization: `Bearer ${token}`,
      },
      body: {
        name,
        assignment_type,
        available_from,
        due_date,
        ...(description && { description }),
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
 * Bulk updates `available_from` and/or `due_date` for multiple assignments.
 * Only accessible by lecturers/admins assigned to the module.
 *
 * @param {object} options
 * @param {number} options.moduleId - Module ID
 * @param {Array<number>} options.assignment_ids - Array of assignment IDs
 * @param {string} [options.available_from] - Optional ISO datetime
 * @param {string} [options.due_date] - Optional ISO datetime
 */
Cypress.Commands.add('apiBulkUpdateAssignments', ({
  moduleId,
  assignment_ids,
  available_from,
  due_date,
}) => {
  return cy.window().then((win) => {
    const token = getAuthToken.call(win);
    if (!token) throw new Error("Missing auth token");

    const body = { assignment_ids };
    if (available_from) body.available_from = available_from;
    if (due_date) body.due_date = due_date;

    return cy.request({
      method: 'PUT',
      url: `${API_BASE_URL}/modules/${moduleId}/assignments/bulk`,
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
 * Deletes a single assignment from a module (admin or lecturer access required).
 *
 * @param {object} options
 * @param {number} options.moduleId - Module ID
 * @param {number} options.assignmentId - Assignment ID
 */
Cypress.Commands.add('apiDeleteAssignment', ({ moduleId, assignmentId }) => {
  return cy.window().then((win) => {
    const token = getAuthToken.call(win);
    if (!token) throw new Error('Missing auth token');

    return cy.request({
      method: 'DELETE',
      url: `${API_BASE_URL}/modules/${moduleId}/assignments/${assignmentId}`,
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
 * Bulk deletes multiple assignments in a module (admin or lecturer access).
 *
 * @param {object} options
 * @param {number} options.moduleId - Module ID
 * @param {Array<number>} options.assignmentIds - Assignment IDs to delete
 */
Cypress.Commands.add('apiBulkDeleteAssignments', ({ moduleId, assignmentIds }) => {
  return cy.window().then((win) => {
    const token = getAuthToken.call(win);
    if (!token) throw new Error('Missing auth token');

    return cy.request({
      method: 'DELETE',
      url: `${API_BASE_URL}/modules/${moduleId}/assignments/bulk`,
      headers: {
        Authorization: `Bearer ${token}`,
      },
      body: {
        assignment_ids: assignmentIds,
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
 * Transitions an assignment to `Open` status.
 * Only allowed if current status is Ready, Closed, or Archived.
 *
 * @param {object} options
 * @param {number} options.moduleId - Module ID
 * @param {number} options.assignmentId - Assignment ID
 */
Cypress.Commands.add('apiOpenAssignment', ({ moduleId, assignmentId }) => {
  return cy.window().then((win) => {
    const token = getAuthToken.call(win);
    if (!token) throw new Error("Missing auth token");

    return cy.request({
      method: 'PUT',
      url: `${API_BASE_URL}/modules/${moduleId}/assignments/${assignmentId}/open`,
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
 * Transitions an assignment to `Closed` status.
 * Only allowed if current status is Open.
 *
 * @param {object} options
 * @param {number} options.moduleId - Module ID
 * @param {number} options.assignmentId - Assignment ID
 */
Cypress.Commands.add('apiCloseAssignment', ({ moduleId, assignmentId }) => {
  return cy.window().then((win) => {
    const token = getAuthToken.call(win);
    if (!token) throw new Error("Missing auth token");

    return cy.request({
      method: 'PUT',
      url: `${API_BASE_URL}/modules/${moduleId}/assignments/${assignmentId}/close`,
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
 * Retrieves readiness report for a specific assignment.
 * Reports which setup files/components are present and if the assignment is ready.
 *
 * @param {object} options
 * @param {number} options.moduleId - Module ID
 * @param {number} options.assignmentId - Assignment ID
 * @returns {Promise<{status: number, body: object}>}
 */
Cypress.Commands.add('apiGetAssignmentReadiness', ({ moduleId, assignmentId }) => {
  return cy.window().then((win) => {
    const token = getAuthToken.call(win);
    if (!token) throw new Error('Missing auth token');

    return cy.request({
      method: 'GET',
      url: `${API_BASE_URL}/modules/${moduleId}/assignments/${assignmentId}/readiness`,
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
 * Fully sets up an assignment via API:
 * - Uploads config, main, memo, and makefile zips
 * - Creates 3 default tasks
 * - Generates memo output and mark allocator
 *
 * @param {Object} params
 * @param {number} params.moduleId
 * @param {number} params.assignmentId
 */
Cypress.Commands.add('apiSetupAssignment', ({ moduleId, assignmentId }) => {
  const upload = (fileType, fixture) => {
    return cy.apiUploadAssignmentFile({
      moduleId,
      assignmentId,
      fileType,
      fixturePath: fixture,
    }).then((res) => {
      expect(res.status).to.eq(201);
      expect(res.body.success).to.be.true;
    });
  };

  const createTask = (index) => {
    return cy.apiCreateAssignmentTask({
      moduleId,
      assignmentId,
      name: `Task ${index}`,
      task_number: index,
      command: `make task${index}`,
    }).then((res) => {
      expect(res.status).to.eq(201);
      expect(res.body.success).to.be.true;
    });
  };

  // Step 1: Upload required files
  upload('config', 'config.json');
  upload('main', 'java_main.zip');
  upload('memo', 'java_memo.zip');
  upload('makefile', 'java_makefile.zip');

  // Step 2: Create 3 default tasks
  createTask(1);
  createTask(2);
  createTask(3);

  // Step 3: Generate memo output
  cy.apiGenerateMemoOutput({ moduleId, assignmentId }).then((res) => {
    expect(res.status).to.eq(200);
    expect(res.body.success).to.be.true;
  });

  // Step 4: Generate mark allocator
  cy.apiGenerateMarkAllocator({ moduleId, assignmentId }).then((res) => {
    expect(res.status).to.eq(200);
    expect(res.body.success).to.be.true;
  });
});
