describe("Lecturer: Assignment Operations", () => {
  const timestamp = Date.now();

  let mod = {
    id: null,
    code: `MOD${timestamp.toString().slice(-3)}`,
    year: new Date().getFullYear(),
    description: 'Module for assignment tests',
    credits: 12,
  }

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

    cy.createModule({
      code: mod.code,
      year: mod.year,
      description: mod.description,
      credits: mod.credits,
    }).then(({ moduleId }) => {
      mod.id = moduleId;

      cy.assignUserToModule({
        moduleId: mod.id,
        role: "lecturer",
        search: "lecturer",
      });
    });
  });

  beforeEach(() => {
    cy.session('lecturer', () => {
      cy.loginAs('lecturer');
    });
  });

  after(() => {
    cy.loginAs('admin');
    // Cleanup: Delete module
    if (mod.id) {
      cy.deleteModule({ moduleId: mod.id, search: mod.code });
    }
  });

  it('can open the assignments page for the created module', () => {
    cy.visit('/modules');
    cy.get('[data-cy="entity-search"]').type(`${mod.code}{enter}`);
    cy.get(`[data-cy="entity-${mod.id}"]`).should('exist').click();
    cy.contains('Assignments').click();
    cy.url().should('include', `/modules/${mod.id}/assignments`);
  });

  it('can create a new assignment and verify it is in the table', () => {
    cy.createAssignment({
      moduleId: mod.id,
      name: assignment.name,
      description: assignment.description,
      availableFrom: assignment.availableFrom,
      dueDate: assignment.dueDate,
      expectedStatus: 200,
    }).then(({ assignmentId }) => {
      assignment.id = assignmentId;
      cy.get('[data-cy="entity-search"]').clear().type(`${assignment.name}{enter}`);
      cy.get(`[data-cy="entity-${assignment.id}"]`).should('exist');
    });
  });

  it('can setup an assignment', () => {
    cy.setupAssignment({moduleId: mod.id, assignmentId: assignment.id});
  });

  it('can edit the previously created assignment', () => {
    expect(assignment.id, 'assignmentId should be set').to.be.a('number');

    // Update local state
    assignment.name = 'Updated Assignment';
    assignment.description = 'Updated via Cypress';

    cy.editAssignment({
      moduleId: mod.id,
      assignmentId: assignment.id,
      search: 'Test Assignment',
      name: assignment.name,
      description: assignment.description,
      availableFrom: assignment.availableFrom,
      dueDate: assignment.dueDate,
      expectedStatus: 200,
    }).then(() => {
      cy.contains('Assignment updated successfully').should('exist');
    });
  });

  it('can delete the previously created assignment', () => {
    expect(assignment.id, 'assignmentId should be set').to.be.a('number');

    cy.deleteAssignment({
      moduleId: mod.id,
      assignmentId: assignment.id,
      search: assignment.name,
    });

    cy.get(`[data-cy="entity-${assignment.id}"]`).should('not.exist');
  });
});