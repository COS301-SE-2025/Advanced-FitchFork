describe("Tutor Route Access", () => {
  const timestamp = Date.now();
  const testModule = {
    id: null,
    code: `TUT${timestamp.toString().slice(-3)}`,
    year: new Date().getFullYear(),
    description: "Access restriction test",
    credits: 12,
  };

  let tutorId;

  before(() => {
    // Admin login session
    cy.session("admin", () => {
      cy.apiLoginAs("admin");
    });

    // Create the module
    cy.apiCreateModule(testModule).then((res) => {
      expect(res.status).to.eq(201);
      testModule.id = res.body.data.id;

      // Login as tutor to get user ID
      return cy.apiLoginAs("tutor")
        .then(() => cy.apiGetCurrentUser())
        .then((res) => {
          expect(res.status).to.eq(200);
          tutorId = res.body.data.id;

          // Assign tutor to module as admin
          return cy.apiLoginAs("admin").then(() =>
            cy.apiAssignPersonnel({
              moduleId: testModule.id,
              userIds: [tutorId],
              role: "tutor",
            }).then((res) => {
              expect(res.status).to.eq(200);
            })
          );
        });
    });

    // Define tutor session
    cy.session("tutor", () => {
      cy.apiLoginAs("tutor");
    });
  });

  beforeEach(() => {
    cy.session("tutor", () => {
      cy.apiLoginAs("tutor");
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
    cy.apiLoginAs("admin").then(() => {
      if (testModule.id) {
        cy.apiDeleteModule({ moduleId: testModule.id }).then((res) => {
          expect(res.status).to.eq(200);
        });
      }
    });
  });
});
