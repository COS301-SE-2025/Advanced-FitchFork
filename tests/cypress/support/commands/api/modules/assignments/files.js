import { getAuthToken } from "@utils/auth";
import { API_BASE_URL } from "@utils/api";

/**
 * Uploads a single file to an assignment using multipart/form-data.
 * Requires lecturer/admin token and a file from fixtures.
 *
 * @param {object} options
 * @param {number} options.moduleId - The module ID
 * @param {number} options.assignmentId - The assignment ID
 * @param {string} options.fileType - The type of file (e.g., "main", "memo", "spec")
 * @param {string} options.fixturePath - The relative path to the fixture file (e.g., "main.rs")
 * @returns {Promise<{status: number, body: object}>}
 */
Cypress.Commands.add('apiUploadAssignmentFile', ({ moduleId, assignmentId, fileType, fixturePath }) => {
  const isJson = fixturePath.endsWith('.json');

  return cy.fixture(fixturePath, isJson ? 'utf8' : 'binary')
    .then((fileContent) => {
      const fileBlob = isJson
        ? new Blob([typeof fileContent === 'string' ? fileContent : JSON.stringify(fileContent)], {
            type: 'application/json',
          })
        : Cypress.Blob.binaryStringToBlob(fileContent);

      const formData = new FormData();
      formData.append('file_type', fileType);
      formData.append('file', fileBlob, fixturePath.split('/').pop());

      return cy.window().then((win) => {
        const token = getAuthToken.call(win);
        if (!token) throw new Error('Missing auth token');

        return fetch(`${API_BASE_URL}/modules/${moduleId}/assignments/${assignmentId}/files`, {
          method: 'POST',
          headers: {
            Authorization: `Bearer ${token}`,
          },
          body: formData,
        });
      });
    })
    .then((response) =>
      response.json().then((body) =>
        cy.wrap({
          status: response.status,
          body,
        })
      )
    );
});

/**
 * Deletes one or more uploaded files from a specific assignment.
 * Requires lecturer/admin token.
 *
 * @param {object} options
 * @param {number} options.moduleId - The module ID
 * @param {number} options.assignmentId - The assignment ID
 * @param {number[]} options.fileIds - Array of file IDs to delete (non-empty)
 * @returns {Promise<{status: number, body: object}>}
 */
Cypress.Commands.add('apiDeleteAssignmentFiles', ({ moduleId, assignmentId, fileIds }) => {
  if (!Array.isArray(fileIds) || fileIds.length === 0) {
    throw new Error('fileIds must be a non-empty array of IDs');
  }

  return cy.window().then((win) => {
    const token = getAuthToken.call(win);
    if (!token) throw new Error('Missing auth token');

    return fetch(`/api/modules/${moduleId}/assignments/${assignmentId}/files`, {
      method: 'DELETE',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${token}`
      },
      body: JSON.stringify({ file_ids: fileIds })
    });
  }).then((response) =>
    response.json().then((body) =>
      cy.wrap({
        status: response.status,
        body
      })
    )
  );
});
