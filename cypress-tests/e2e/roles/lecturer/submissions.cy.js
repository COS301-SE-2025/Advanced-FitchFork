describe("Admin: Submission Operations", () => {
  const currentYear = new Date().getFullYear();
  const timestamp = Date.now();

  let module = {
    id: null,
    code: `MOD${timestamp.toString().slice(-3)}`,
    year: currentYear,
    description: 'Valid code test',
    credits: 16,
  };

  let assignment = {
    id: null,
    name: 'Test Assignment',
    description: 'Created via Cypress',
    availableFrom: '2025-07-27 09:00',
    dueDate: '2025-07-30 17:00',
  };

  before(() => {
    cy.session('admin', () => {
      cy.loginAs('admin');
    });

    return cy.createModule({
      code: module.code,
      year: module.year,
      description: module.description,
      credits: module.credits,
      expectedStatus: 201,
    }).then(({ moduleId: id }) => {
      module.id = id;

      cy.assignUserToModule({
        moduleId: module.id,
        role: "lecturer",
        search: "lecturer",
      });

      cy.createAssignment({
        moduleId: module.id,
        name: assignment.name,
        description: assignment.description,
        availableFrom: assignment.availableFrom,
        dueDate: assignment.dueDate,
        expectedStatus: 200,
      });
    }).then(({ assignmentId }) => {
      assignment.id = assignmentId;

      cy.setupAssignment({moduleId: module.id, assignmentId: assignment.id});
    });
  });

  beforeEach(() => {
    cy.session('lecturer', () => {
      cy.loginAs('lecturer');
    });
  })

  it("can submit assignment", () => {
    cy.submitAssignment({
      moduleId: module.id, 
      assignmentId: assignment.id, 
      fixtureFile: "java_submission.zip"
    })
    .then(({submissionId}) => {
      cy.get(`[data-cy="entity-${submissionId}"]`).should('exist');
    });
  })
})