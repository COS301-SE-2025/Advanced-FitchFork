describe("Admin: Submission Operations", () => {
  const timestamp = Date.now();
  const currentYear = new Date().getFullYear();

  const mod = {
    id: null,
    code: `MOD${timestamp.toString().slice(-3)}`,
    year: currentYear,
    description: "Valid code test",
    credits: 16,
  };

  const assignment = {
    id: null,
    name: "Test Assignment",
    description: "Created via Cypress",
    availableFrom: "2025-07-27T09:00:00+02:00",
    dueDate: "2025-07-30T17:00:00+02:00",
    assignment_type: "assignment",
  };


  before(() => {
    cy.session("admin", () => {
      cy.apiLoginAs("admin");
    });

    // Create module
    cy.apiCreateModule(mod).then(({ status, body }) => {
      expect(status).to.eq(201);
      mod.id = body.data.id;

      // Create assignment
      return cy.apiCreateAssignment({
        moduleId: mod.id,
        name: assignment.name,
        description: assignment.description,
        available_from: assignment.availableFrom,
        due_date: assignment.dueDate,
      });
    }).then(({ status, body }) => {
      expect(status).to.eq(201);
      assignment.id = body.data.id;

      // Setup assignment: upload files, tasks, memo & mark allocator
      cy.apiSetupAssignment({
        moduleId: mod.id,
        assignmentId: assignment.id,
      });
    });
  });

  it("can submit assignment", () => {
    cy.submitAssignment({
      moduleId: mod.id,
      assignmentId: assignment.id,
      fixtureFile: "java_submission.zip",
    }).then(({ submissionId }) => {
      cy.get(`[data-cy="entity-${submissionId}"]`).should("exist");
    });
  });
});
