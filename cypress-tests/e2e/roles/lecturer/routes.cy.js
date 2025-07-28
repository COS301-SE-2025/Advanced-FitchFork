describe("Lecturer Route Access", () => {
  beforeEach(() => {
    cy.session("lecturer", () => {
      cy.loginAs("lecturer");
    });
  })

  it("cannot access te users list page", () => {
    cy.visit("/users");
    cy.contains("Unauthorized");
  })
})