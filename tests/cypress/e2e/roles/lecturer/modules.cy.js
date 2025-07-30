describe("Lecturer: Module Operations", () => {
  const timestamp = Date.now();

  const mod = {
    id: null,
    code: `LET${timestamp.toString().slice(-3)}`,
    year: new Date().getFullYear(),
    description: "Lecturer view module test",
    credits: 12,
  };

  let lecturerId;

  before(() => {
    // Start admin session
    cy.session("admin", () => {
      cy.apiLoginAs("admin");
    });

    // Create module as admin
    cy.apiCreateModule(mod).then((res) => {
      expect(res.status).to.eq(201);
      mod.id = res.body.data.id;
    });

    // Login as lecturer (no session) to get ID
    cy.apiLoginAs("lecturer")
      .then(() => cy.apiGetCurrentUser())
      .then((res) => {
        lecturerId = res.body.data.id;
        expect(lecturerId).to.be.a("number");
      });

    // Assign lecturer to module
    cy.apiLoginAs("admin").then(() => {
      cy.apiAssignPersonnel({
        moduleId: mod.id,
        userIds: [lecturerId],
        role: "lecturer",
      }).then((res) => {
        expect(res.status).to.eq(200);
        expect(res.body.success).to.be.true;
      });
    });

    // Define session for lecturer
    cy.session("lecturer", () => {
      cy.apiLoginAs("lecturer");
    });
  });

  beforeEach(() => {
    cy.session("lecturer", () => {
      cy.apiLoginAs("lecturer");
    });
  });

  it("can view a module by clicking on it", () => {
    cy.visit("/modules");
    cy.get('[data-cy="view-toggle-table"]').click();

    cy.get('[data-cy="entity-search"]').clear().type(`${mod.code}{enter}`);
    cy.get(`[data-cy="entity-${mod.id}"]`).should("be.visible").click();

    cy.url().should("include", `/modules/${mod.id}`);
    cy.contains(mod.description);
  });

  it("cannot see the add module button", () => {
    cy.visit("/modules");
    cy.get('[data-cy="view-toggle-table"]').click();
    cy.contains("Add Module").should("not.exist");
  });

  it("cannot see any actions column", () => {
    cy.visit("/modules");
    cy.get('[data-cy="view-toggle-table"]').click();
    cy.contains("Actions").should("not.exist");
  });

  after(() => {
    cy.apiLoginAs("admin").then(() => {
      if (mod.id) {
        cy.apiDeleteModule({ moduleId: mod.id }).then((res) => {
          expect(res.status).to.eq(200);
        });
      }
    });
  });
});
