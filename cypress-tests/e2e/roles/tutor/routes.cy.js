describe("Tutor Route Access", () => {
  const timestamp = Date.now();
  const testModule = {
    code: `TUT${timestamp.toString().slice(-3)}`,
    year: new Date().getFullYear(),
    description: "Access restriction test",
    credits: 12,
  };

  const tutorUsername = "tutor"; // change if your seeded tutor has a different name
  let moduleId;

  before(() => {
    // Create module and assign tutor to it as admin
    cy.session("admin", () => {
      cy.loginAs("admin");
    });

    cy.createModule(testModule).then(({ moduleId: id }) => {
      moduleId = id;
      testModule.id = id;

      // Now assign the tutor to the module via personnel UI
      cy.assignUserToModule({
        moduleId,
        role: "Tutor",
        search: tutorUsername,
      });
    });
  });

  beforeEach(() => {
    cy.session("tutor", () => {
      cy.loginAs("tutor");
    });
  });

  it("cannot access the users list page", () => {
    cy.visit("/users");
    cy.contains("Unauthorized");
  });

  it("cannot access the module personnel page", () => {
    cy.visit(`/modules/${testModule.id}/personnel`);
    cy.contains("Unauthorized");
  });

  after(() => {
    cy.session("admin", () => {
      cy.loginAs("admin");
    });

    cy.deleteModule({ moduleId: testModule.id, search: testModule.code });
  });
});
