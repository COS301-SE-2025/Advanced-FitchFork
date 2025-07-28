describe("Lecturer: Module Personnel Access", () => {
  const timestamp = Date.now();
  const searchPrefix = `personnel_${timestamp}`;

  const mod = {
    code: `MOD${timestamp.toString().slice(-3)}`,
    year: new Date().getFullYear(),
    description: "Personnel assignment test",
    credits: 16,
  };

  const testUsers = [
    { username: `${searchPrefix}_1`, email: `${searchPrefix}_1@test.com` },
    { username: `${searchPrefix}_2`, email: `${searchPrefix}_2@test.com` },
  ];

  before(() => {
    cy.session("admin", () => {
      cy.loginAs("admin");
    });

    cy.wrap(null).then(() =>
      Cypress.Promise.each(testUsers, (user, index) =>
        cy.createUser(user).then(({ userId }) => {
          testUsers[index].userId = userId;
        })
      )
    );

    cy.createModule(mod).then(({ moduleId }) => {
      mod.id = moduleId;

      // Assign the built-in lecturer user
      cy.assignUserToModule({
        moduleId: mod.id,
        role: "lecturer",
        search: "lecturer",
      });
    });
  });

  beforeEach(() => {
    cy.session("lecturer", () => {
      cy.loginAs("lecturer");
    });
  });

  after(() => {
    cy.session("admin", () => {
      cy.loginAs("admin");
    });

    cy.wrap(null).then(() =>
      Cypress.Promise.each(testUsers, ({ userId, username }) =>
        cy.deleteUser({ userId, search: username })
      )
    );

    cy.deleteModule({ moduleId: mod.id, search: mod.code });
  });

  function assignUsersToRole(role) {
    cy.visit(`/modules/${mod.id}/personnel`);
    cy.get('[data-cy="personnel-role-selector"]').contains(role).click();
    cy.get('[data-cy="available-user-search"]').clear().type(searchPrefix).type("{enter}");

    for (const { userId } of testUsers) {
      cy.get(`[data-cy="available-user-row-${userId}"]`).click();
    }

    cy.get(".ant-transfer-operation button").first().click();

    for (const { userId } of testUsers) {
      cy.get(`[data-cy="assigned-user-row-${userId}"]`).should("exist");
    }
  }

  function unassignUsersFromRole(role) {
    cy.visit(`/modules/${mod.id}/personnel`);
    cy.get('[data-cy="personnel-role-selector"]').contains(role).click();

    for (const { userId } of testUsers) {
      cy.get(`[data-cy="assigned-user-row-${userId}"]`).click();
    }

    cy.get(".ant-transfer-operation button").last().click();

    for (const { userId } of testUsers) {
      cy.get(`[data-cy="available-user-row-${userId}"]`).should("not.exist");
    }
  }

  it("can assign users to Student role using search", () => {
    assignUsersToRole("Student");
  });

  it("can unassign users from Student role", () => {
    unassignUsersFromRole("Student");
  });

  it("can assign users to Tutor role using search", () => {
    assignUsersToRole("Tutor");
  });

  it("can unassign users from Tutor role", () => {
    unassignUsersFromRole("Tutor");
  });

  it("cannot see the Lecturer segment, not allowed to assign lecturers", () => {
    cy.visit(`/modules/${mod.id}/personnel`);
    cy.get('[data-cy="personnel-role-selector"]')
      .contains(/^lecturer$/i)
      .should("not.exist");
  });
});
