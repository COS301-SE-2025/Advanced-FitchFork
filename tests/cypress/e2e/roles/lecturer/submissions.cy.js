describe("Lecturer: Submission Operations", () => {
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

  let lecturerId;

  before(() => {
    cy.session('admin', () => {
      cy.apiLoginAs('admin');
    });

    cy.apiCreateModule(module).then((res) => {
      expect(res.status).to.eq(201);
      module.id = res.body.data.id;

      return cy.apiLoginAs('lecturer')
        .then(() => cy.apiGetCurrentUser())
        .then((res) => {
          expect(res.status).to.eq(200);
          lecturerId = res.body.data.id;
          expect(lecturerId).to.be.a('number');

          return cy.apiLoginAs('admin').then(() =>
            cy.apiAssignPersonnel({
              moduleId: module.id,
              userIds: [lecturerId],
              role: 'lecturer',
            }).then((res) => {
              expect(res.status).to.eq(200);
            })
          );
        });
    })
    .then(() => {
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
      return cy.apiSetupAssignment({
        moduleId: module.id,
        assignmentId: assignment.id,
      });
    });
  });

  beforeEach(() => {
    cy.session('lecturer', () => {
      cy.apiLoginAs('lecturer');
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
          expect(res.body.success).to.be.true;
        });
      }
    });
  });
});
