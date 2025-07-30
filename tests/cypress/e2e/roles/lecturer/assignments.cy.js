describe("Lecturer: Assignment Operations", () => {
  const timestamp = Date.now();

  const mod = {
    id: null,
    code: `MOD${timestamp.toString().slice(-3)}`,
    year: new Date().getFullYear(),
    description: 'Module for assignment tests',
    credits: 12,
  };

  let lecturerId;

  // -- Setup: Create module and assign lecturer role
  before(() => {
    // Start admin session
    cy.session('admin', () => {
      cy.apiLoginAs('admin');
    });

    // Create module as admin
    cy.apiCreateModule(mod).then((res) => {
      mod.id = res.body.data.id;
    });

    // Get lecturer user ID via login
    cy.apiLoginAs('lecturer')
      .then(() => cy.apiGetCurrentUser())
      .then((res) => {
        lecturerId = res.body.data.id;
      });

    // Assign lecturer to the module
    cy.apiLoginAs('admin').then(() => {
      cy.apiAssignPersonnel({
        moduleId: mod.id,
        userIds: [lecturerId],
        role: 'lecturer',
      });
    });

    // Define lecturer session once
    cy.session('lecturer', () => {
      cy.apiLoginAs('lecturer');
    });
  });

  // -- Login as lecturer for each test
  beforeEach(() => {
    cy.session('lecturer', () => {
      cy.apiLoginAs('lecturer');
    });
  });

  // -- Cleanup
  after(() => {
    cy.apiLoginAs('admin').then(() => {
      if (mod.id) {
        cy.apiDeleteModule({ moduleId: mod.id });
      }
    });
  });

  // -- Tests
  it('can open the assignments page for the created module', () => {
    cy.visit('/modules');
    cy.get('[data-cy="entity-search"]').clear().type(`${mod.code}{enter}`);
    cy.get(`[data-cy="entity-${mod.id}"]`).should('exist').click();
    cy.contains('Assignments').click();
    cy.url().should('include', `/modules/${mod.id}/assignments`);
  });

  it('can see the add assignment button', () => {
    cy.visit(`/modules/${mod.id}/assignments`);
    cy.get('[data-cy="control-action-create"]').contains('Add Assignment');
  });
});
