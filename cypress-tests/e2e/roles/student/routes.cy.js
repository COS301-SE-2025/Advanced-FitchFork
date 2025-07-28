describe("Student Route Access", () => {
  const timestamp = Date.now();
  const testModule = {
    code: `STU${timestamp.toString().slice(-3)}`,
    year: new Date().getFullYear(),
    description: "Access restriction test",
    credits: 12,
  };

  const studentUsername = "student"; // change if your seeded student has a different name
  let moduleId;

  before(() => {
    // Create module and assign student to it as admin
    cy.session("admin", () => {
      cy.loginAs("admin");
    });

    cy.createModule(testModule).then(({ moduleId: id }) => {
      moduleId = id;
      testModule.id = id;

      // Now assign the student to the module via personnel UI
      cy.assignUserToModule({
        moduleId,
        role: "Student",
        search: studentUsername,
      });
    });
  });

  beforeEach(() => {
    cy.session("student", () => {
      cy.loginAs("student");
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
