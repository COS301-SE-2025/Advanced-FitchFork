import { getAuthToken } from "@utils/auth";

/**
 * Submits a file to an assignment as a student.
 *
 * @param {object} options
 * @param {number} options.moduleId - The module ID
 * @param {number} options.assignmentId - The assignment ID
 * @param {string} options.fixturePath - Path to the file in Cypress fixtures (e.g., "submissions/solution.zip")
 * @param {boolean} [options.isPractice=false] - Whether this is a practice submission
 * @returns {Promise<{ status: number, body: object }>}
 */
Cypress.Commands.add('apiSubmitAssignment', ({ moduleId, assignmentId, fixturePath, isPractice = false }) => {
  return cy.fixture(fixturePath, 'binary')
    .then(Cypress.Blob.binaryStringToBlob)
    .then((fileBlob) => {
      const formData = new FormData();
      formData.append('file', fileBlob, fixturePath.split('/').pop());
      if (isPractice) formData.append('is_practice', 'true');

      return cy.window().then((win) => {
        const token = getAuthToken.call(win);
        if (!token) throw new Error('Missing auth token');

        return fetch(`/api/modules/${moduleId}/assignments/${assignmentId}/submissions`, {
          method: 'POST',
          headers: {
            Authorization: `Bearer ${token}`
          },
          body: formData
        });
      });
    })
    .then(async (response) => {
      const body = await response.json();
      return cy.wrap({
        status: response.status,
        body
      });
    });
});
