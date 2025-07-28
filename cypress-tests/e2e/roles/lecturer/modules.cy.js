describe("Lecturer: Module Operations", () => {
  const timestamp = Date.now();
  const mod = {
    id: null,
    code: `LET${timestamp.toString().slice(-3)}`,
    year: new Date().getFullYear(),
    description: "Lecturer view module test",
    credits: 12,
  };

  before(() => {
    // Admin creates the module and assigns lecturer to it
    cy.session('admin', () => {
      cy.loginAs('admin');
    });

    cy.createModule(mod).then(({ moduleId }) => {
      mod.id = moduleId;

      // Assign lecturer to this module
      cy.assignUserToModule({
        moduleId: mod.id,
        role: "Lecturer",
        search: "lecturer",
      });
    });
  });

  beforeEach(() => {
    cy.session('lecturer', () => {
      cy.loginAs('lecturer');
    });
  });

  it("can view a module by clicking on it", () => {
    cy.visit("/modules");
    cy.get('[data-cy="view-toggle-table"]').click();

    // Search for the specific module code
    cy.get('[data-cy="entity-search"]').clear().type(`${mod.code}{enter}`);
    
    // Select the row directly by known ID
    cy.get(`[data-cy="entity-${mod.id}"]`)
      .should('be.visible')
      .click();

    // Confirm we're in the correct module page
    cy.url().should('include', `/modules/${mod.id}`);
    cy.contains(mod.description);
  });

  it("cannot see the add module button", () => {
    cy.visit("/modules");
    cy.get('[data-cy="view-toggle-table"]').click();
    cy.contains("Add Module").should('not.exist');
  });

  it("cannot see any actions column", () => {
    cy.visit("/modules");
    cy.get('[data-cy="view-toggle-table"]').click();
    cy.contains("Actions").should('not.exist');
  });

  after(() => {
    cy.visit("/");
    cy.loginAs('admin');
    cy.deleteModule({ moduleId: mod.id, search: mod.code });
  });
});
