describe("Admin: Assignment Operations", () => {
  let moduleId;
  let moduleCode;
  const year = new Date().getFullYear();

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

    const timestamp = Date.now();
    moduleCode = `MOD${timestamp.toString().slice(-3)}`;

    cy.createModule({
      code: moduleCode,
      year,
      description: 'Module for assignment tests',
      credits: 12,
    }).then(({ moduleId: id }) => {
      moduleId = id;

      cy.assignUserToModule({
        moduleId: moduleId,
        role: "tutor",
        search: "tutor",
      });
    });
  });

  beforeEach(() => {
    cy.session('tutor', () => {
      cy.loginAs('tutor');
    });
  });

  after(() => {
    cy.session('admin', () => {
      cy.loginAs('admin');
    });
    // Cleanup: Delete module
    if (moduleId) {
      cy.deleteModule({ moduleId, search: moduleCode });
    }
  });

  it('can open the assignments page', () => {
    cy.visit('/modules');
    cy.get('[data-cy="view-toggle-table"]').click();
    cy.get('[data-cy="entity-search"]').type(`${moduleCode}{enter}`);
    cy.get(`[data-cy="entity-${moduleId}"]`).should('exist').click();
    cy.contains('Assignments').click();
    cy.url().should('include', `/modules/${moduleId}/assignments`);
  });

  it('cannot see add assignment button ', () => {
    cy.visit(`/modules/${moduleId}/assignments`);
    cy.get('[data-cy="view-toggle-table"]').click();

    cy.contains('Add Assignment').should('not.exist');
  });

  it('cannot see actions column ', () => {
    cy.visit(`/modules/${moduleId}/assignments`);
    cy.get('[data-cy="view-toggle-table"]').click();

    cy.contains('Actions').should('not.exist');
  });
});