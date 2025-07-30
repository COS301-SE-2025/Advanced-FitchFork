describe("Lecturer Route Access", () => {
  beforeEach(() => {
    cy.session("lecturer", () => {
      cy.apiLoginAs("lecturer");
    });
  });

  it("cannot access the users list page", () => {
    cy.visit("/users");
    cy.contains("Unauthorized");
  });
});
