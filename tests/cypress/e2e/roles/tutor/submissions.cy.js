describe("Tutor: Submission Operations", () => {
  const currentYear = new Date().getFullYear();
  const timestamp = Date.now();

  const module = {
    id: null,
    code: `MOD${timestamp.toString().slice(-3)}`,
    year: currentYear,
    description: 'Valid code test',
    credits: 16,
  };

  const assignment = {
    id: null,
    name: 'Test Assignment',
    description: 'Created via Cypress',
    available_from: '2025-07-27T09:00:00Z',
    due_date: '2025-07-30T17:00:00Z',
  };

  let tutorId;

  before(() => {
    // Start admin session
    cy.session('admin', () => {
      cy.apiLoginAs('admin');
    });

    // Create module
    cy.apiCreateModule(module).then((res) => {
      expect(res.status).to.eq(201);
      module.id = res.body.data.id;

      // Login as tutor and get ID
      return cy.apiLoginAs('tutor')
        .then(() => cy.apiGetCurrentUser())
        .then((res) => {
          expect(res.status).to.eq(200);
          tutorId = res.body.data.id;
          expect(tutorId).to.be.a('number');

          // Assign tutor to module
          return cy.apiLoginAs('admin').then(() =>
            cy.apiAssignPersonnel({
              moduleId: module.id,
              userIds: [tutorId],
              role: 'tutor',
            }).then((res) => {
              expect(res.status).to.eq(200);
            })
          );
        });
    })
    .then(() => {
      // Create assignment
      return cy.apiCreateAssignment({
        moduleId: module.id,
        name: assignment.name,
        description: assignment.description,
        available_from: assignment.available_from,
        due_date: assignment.due_date,
      }).then((res) => {
        expect(res.status).to.eq(201);
        assignment.id = res.body.data.id;
      });
    })
    .then(() => {
      // Setup assignment
      return cy.apiSetupAssignment({
        moduleId: module.id,
        assignmentId: assignment.id,
      });
    });

    // Define tutor session
    cy.session('tutor', () => {
      cy.apiLoginAs('tutor');
    });
  });

  beforeEach(() => {
    cy.session('tutor', () => {
      cy.apiLoginAs('tutor');
    });
  });

  it("can submit assignment", () => {
    cy.submitAssignment({
      moduleId: module.id,
      assignmentId: assignment.id,
      fixtureFile: "java_submission.zip",
    }).then(({ submissionId }) => {
      cy.get(`[data-cy="entity-${submissionId}"]`).should('exist');
    });
  });

  after(() => {
    cy.apiLoginAs('admin').then(() => {
      if (module.id) {
        cy.apiDeleteModule({ moduleId: module.id }).then((res) => {
          expect(res.status).to.eq(200);
        });
      }
    });
  });
});
