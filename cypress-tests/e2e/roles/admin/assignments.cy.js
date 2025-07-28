describe("Admin: Assignment Operations", () => {
  const timestamp = Date.now();

  let mod = {
    id: null,
    code: `MOD${timestamp.toString().slice(-3)}`,
    year:  new Date().getFullYear(),
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

    cy.createModule(mod).then(({ moduleId }) => {
      mod.id = moduleId;
    });
  });

  beforeEach(() => {
    cy.session('admin', () => {
      cy.loginAs('admin');
    });
  });

  after(() => {
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

  it("can bulk edit multiple assignments' due dates", () => {
    const newDueDate = '2025-08-05 23:59';
    const seedAssignments = ['Bulk A1', 'Bulk A2', 'Bulk A3'];
    const assignments = [];

    cy.wrap(null)
      .then(() =>
        Cypress.Promise.each(seedAssignments, (name, i) =>
          cy
            .createAssignment({
              moduleId: mod.id,
              name,
              description: `Bulk assignment ${i}`,
              availableFrom: '2025-07-29 08:00',
              dueDate: '2025-08-01 23:59',
            })
            .then(({ assignmentId }) => {
              assignments.push({ name, id: assignmentId });
            })
        )
      )
      .then(() => {
        cy.visit(`/modules/${mod.id}/assignments`);
        cy.get('[data-cy="view-toggle-table"]').click();

        // Select checkboxes after DOM is fully loaded
        cy.wrap(assignments).each(({ id }) => {
          cy.get(`[data-cy="entity-${id}"]`)
            .find('input[type="checkbox"]')
            .check({ force: true });
        });

        // Open bulk edit modal
        cy.get('[data-cy="bulk-action-bulk-edit"]').click();
        cy.get('[id="due_date"]').clear().type(newDueDate);
        cy.contains('OK').click({ force: true });
        cy.get('[data-cy="edit-modal-submit"]').click();

        // Confirm update
        cy.contains('Updated 3/3 assignments').should('exist');

        // Verify due dates updated
        cy.wrap(assignments).each(({ id }) => {
          cy.get(`[data-cy="entity-${id}"]`).within(() => {
            cy.contains('2025-08-05').should('exist');
          });
        });
      });
  });

  it("can bulk delete multiple assignments", () => {
    const seedAssignments = ["Bulk D1", "Bulk D2"];
    const assignments = [];

    cy.wrap(null)
      .then(() =>
        Cypress.Promise.each(seedAssignments, (name, i) =>
          cy
            .createAssignment({
              moduleId: mod.id,
              name,
              description: `To be deleted ${i}`,
              availableFrom: "2025-07-29 10:00",
              dueDate: "2025-08-01 20:00",
            })
            .then(({ assignmentId }) => {
              assignments.push({ name, id: assignmentId });
            })
        )
      )
      .then(() => {
        cy.visit(`/modules/${mod.id}/assignments`);
        cy.get('[data-cy="view-toggle-table"]').click();

        // Select checkboxes for the assignments
        cy.wrap(assignments).each(({ id }) => {
          cy.get(`[data-cy="entity-${id}"]`)
            .find('input[type="checkbox"]')
            .check({ force: true });
        });

        // Open the bulk action dropdown
        cy.get('[data-cy="bulk-action-dropdown"]').should('be.visible').trigger('mouseover');

        // Click the delete action from dropdown
        cy.get('[data-cy="bulk-action-bulk-delete"]').click();

        // Confirm deletion
        cy.contains(`Delete ${assignments.length} selected assignments?`).should("exist");
        cy.contains("Yes").click();

        // Verify deletion success message
        cy.contains(`Deleted ${assignments.length}/${assignments.length} assignments`).should("exist");

        // Ensure assignments are gone
        cy.wrap(assignments).each(({ id }) => {
          cy.get(`[data-cy="entity-${id}"]`).should("not.exist");
        });
      });
  });

  it('cannot set due date before available_from', () => {
    expect(assignment.id, 'assignmentId should be set').to.be.a('number');

    cy.editAssignment({
      moduleId: mod.id,
      assignmentId: assignment.id,
      search: assignment.name,
      name: 'Invalid Date Assignment',
      description: 'Trying invalid dates',
      availableFrom: '2025-08-01 10:00',
      dueDate: '2025-07-30 12:00',
      expectedStatus: 400,
    });

    cy.contains('Due date cannot be before Available From date').should('exist');
    cy.get('[data-cy="edit-modal-submit"]').should('exist');
  });

  it('cannot create an assignment with due date before available from date', () => {
    cy.createAssignment({
      moduleId: mod.id,
      name: 'Invalid Assignment',
      description: 'This should not be allowed',
      availableFrom: '2025-07-30 10:00',
      dueDate: '2025-07-28 10:00',
      expectedStatus: 400,
    });

    cy.contains('Due date cannot be before Available From date').should('exist');
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