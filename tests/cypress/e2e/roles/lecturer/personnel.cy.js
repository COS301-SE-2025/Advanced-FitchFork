describe("Lecturer: Module Personnel Access", () => {
  const timestamp = Date.now();
  const searchPrefix = `personnel_${timestamp}`;

  const mod = {
    id: null,
    code: `MOD${timestamp.toString().slice(-3)}`,
    year: new Date().getFullYear(),
    description: "Personnel assignment test",
    credits: 16,
  };

  const testUsers = [
    { username: `${searchPrefix}_1`, email: `${searchPrefix}_1@test.com`, userId: null },
    { username: `${searchPrefix}_2`, email: `${searchPrefix}_2@test.com`, userId: null },
  ];

  before(() => {
    // Start admin session
    cy.session("admin", () => {
      cy.apiLoginAs("admin");
    });

    // Create users
    cy.wrap(null).then(() =>
      Cypress.Promise.each(testUsers, (user, index) =>
        cy.apiCreateUser({
          username: user.username,
          email: user.email,
          password: "password123",
        }).then(({ userId }) => {
          testUsers[index].userId = userId;
        })
      )
    );

    // Create module and assign lecturer
    cy.apiCreateModule(mod).then((res) => {
      mod.id = res.body.data.id;
    });

    // Assign built-in lecturer to the module
    cy.apiLoginAs("lecturer")
      .then(() => cy.apiGetCurrentUser())
      .then((res) => {
        const lecturerId = res.body.data.id;

        return cy.apiLoginAs("admin").then(() =>
          cy.apiAssignPersonnel({
            moduleId: mod.id,
            userIds: [lecturerId],
            role: "lecturer",
          })
        );
      });

    // Define lecturer session
    cy.session("lecturer", () => {
      cy.apiLoginAs("lecturer");
    });
  });

  beforeEach(() => {
    cy.session("lecturer", () => {
      cy.apiLoginAs("lecturer");
    });
  });

  after(() => {
    cy.apiLoginAs("admin").then(() => {
      Cypress.Promise.each(testUsers, ({ userId, username }) =>
        cy.apiDeleteUser({ userId, search: username })
      );

      if (mod.id) {
        cy.apiDeleteModule({moduleId: mod.id}).then((res) => {
          expect(res.status).to.eq(200);
        });
      }
    });
  });

  it("cannot see the Lecturer segment, not allowed to assign lecturers", () => {
    cy.visit(`/modules/${mod.id}/personnel`);
    cy.get('[data-cy="personnel-role-selector"]')
      .contains(/^lecturer$/i)
      .should("not.exist");
  });
});
