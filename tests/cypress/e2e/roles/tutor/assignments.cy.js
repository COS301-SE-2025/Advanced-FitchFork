describe("Tutor: Assignment Operations", () => {
  const timestamp = Date.now();
  const year = new Date().getFullYear();

  const module = {
    id: null,
    code: `MOD${timestamp.toString().slice(-3)}`,
    year,
    description: 'Module for assignment tests',
    credits: 12,
  };

  let tutorId;

  before(() => {
    // Admin session
    cy.session('admin', () => {
      cy.apiLoginAs('admin');
    });

    // Create module
    cy.apiCreateModule(module).then((res) => {
      expect(res.status).to.eq(201);
      module.id = res.body.data.id;
    });

    // Login as tutor to get ID
    cy.apiLoginAs('tutor')
      .then(() => cy.apiGetCurrentUser())
      .then((res) => {
        expect(res.status).to.eq(200);
        tutorId = res.body.data.id;
        expect(tutorId).to.be.a('number');

        // Re-login as admin and assign tutor
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

    // Define tutor session for test runtime
    cy.session('tutor', () => {
      cy.apiLoginAs('tutor');
    });
  });

  beforeEach(() => {
    cy.session('tutor', () => {
      cy.apiLoginAs('tutor');
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

  it('can open the assignments page', () => {
    cy.visit('/modules');
    cy.get('[data-cy="view-toggle-table"]').click();
    cy.get('[data-cy="entity-search"]').type(`${module.code}{enter}`);
    cy.get(`[data-cy="entity-${module.id}"]`).should('exist').click();
    cy.contains('Assignments').click();
    cy.url().should('include', `/modules/${module.id}/assignments`);
  });

  it('cannot see add assignment button', () => {
    cy.visit(`/modules/${module.id}/assignments`);
    cy.get('[data-cy="view-toggle-table"]').click();
    cy.contains('Add Assignment').should('not.exist');
  });

  it('cannot see actions column', () => {
    cy.visit(`/modules/${module.id}/assignments`);
    cy.get('[data-cy="view-toggle-table"]').click();
    cy.contains('Actions').should('not.exist');
  });
});
