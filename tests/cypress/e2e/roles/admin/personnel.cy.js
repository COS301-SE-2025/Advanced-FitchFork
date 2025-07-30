describe("Admin: Module Personnel Operations", () => {
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
    { id: null, username: `${searchPrefix}_1`, email: `${searchPrefix}_1@test.com`, password: "password123" },
    { id: null, username: `${searchPrefix}_2`, email: `${searchPrefix}_2@test.com`, password: "password123" },
  ];

  before(() => {
    cy.session('admin', () => {
      cy.apiLoginAs("admin");
    })

    // Create users
    cy.wrap(null).then(() =>
      Cypress.Promise.each(testUsers, (user, index) =>
        cy.apiCreateUser(user).then(({ status, body }) => {
          expect(status).to.eq(201);
          testUsers[index].id = body.data.id;
        })
      )
    );

    // Create module
    cy.apiCreateModule(mod).then(({ status, body }) => {
      expect(status).to.eq(201);
      mod.id = body.data.id;
    });
  });

  beforeEach(() => {
    cy.session('admin', () => {
      cy.apiLoginAs("admin");
    })
  });

  after(() => {
    cy.session('admin', () => {
      cy.apiLoginAs("admin");
    })

    // Delete users
    cy.wrap(null).then(() =>
      Cypress.Promise.each(testUsers, ({ id }) =>
        cy.apiDeleteUser(id).then(({ status }) => {
          expect(status).to.eq(200);
        })
      )
    );

    // Delete module
    cy.apiDeleteModule({moduleId: mod.id}).then(({ status }) => {
      expect(status).to.eq(200);
    });
  });

  function assignUsersToRole(role) {
    cy.visit(`/modules/${mod.id}/personnel`);
    cy.get('[data-cy="personnel-role-selector"]').contains(role).click();
    cy.get('[data-cy="available-user-search"]').clear().type(searchPrefix).type("{enter}");

    for (const { id } of testUsers) {
      cy.get(`[data-cy="available-user-row-${id}"]`).click();
    }

    cy.get(".ant-transfer-operation button").first().click();

    for (const { id } of testUsers) {
      cy.get(`[data-cy="assigned-user-row-${id}"]`).should("exist");
    }
  }

  function unassignUsersFromRole(role) {
    cy.visit(`/modules/${mod.id}/personnel`);
    cy.get('[data-cy="personnel-role-selector"]').contains(role).click();

    for (const { id } of testUsers) {
      cy.get(`[data-cy="assigned-user-row-${id}"]`).click();
    }

    cy.get(".ant-transfer-operation button").last().click();

    for (const { id } of testUsers) {
      cy.get(`[data-cy="available-user-row-${id}"]`).should("not.exist");
    }
  }

  it("can assign users to Student role using search", () => {
    assignUsersToRole("Student");
  });

  it("can unassign users from Student role", () => {
    unassignUsersFromRole("Student");
  });

  it("can assign users to Lecturer role using search", () => {
    assignUsersToRole("Lecturer");
  });

  it("can unassign users from Lecturer role", () => {
    unassignUsersFromRole("Lecturer");
  });

  it("can assign users to Tutor role using search", () => {
    assignUsersToRole("Tutor");
  });

  it("can unassign users from Tutor role", () => {
    unassignUsersFromRole("Tutor");
  });
});
