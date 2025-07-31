describe("Student Route Access", () => {
  const timestamp = Date.now();
  const testModule = {
    id: null,
    code: `STU${timestamp.toString().slice(-3)}`,
    year: new Date().getFullYear(),
    description: "Access restriction test",
    credits: 12,
  };

  const studentUsername = "student";
  let studentId;

  before(() => {
    // Admin logs in and creates module
    cy.session("admin", () => {
      cy.apiLoginAs("admin");
    });

    cy.apiCreateModule(testModule).then((res) => {
      expect(res.status).to.eq(201);
      testModule.id = res.body.data.id;
    });

    // Login as student and fetch ID
    cy.apiLoginAs("student")
      .then(() => cy.apiGetCurrentUser())
      .then((res) => {
        expect(res.status).to.eq(200);
        studentId = res.body.data.id;
        expect(studentId).to.be.a("number");

        // Re-login as admin and assign student to the module
        return cy.apiLoginAs("admin").then(() =>
          cy.apiAssignPersonnel({
            moduleId: testModule.id,
            userIds: [studentId],
            role: "student",
          }).then((res) => {
            expect(res.status).to.eq(200);
          })
        );
      });

    // Define student session for use in tests
    cy.session("student", () => {
      cy.apiLoginAs("student");
    });
  });

  beforeEach(() => {
    cy.session("student", () => {
      cy.apiLoginAs("student");
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
          expect(res.body.success).to.be.true;
        });
      }
    });
  });
});
