describe("Admin: Module Operations", () => {
  const currentYear = new Date().getFullYear();
  const timestamp = Date.now();

  let mod = {
    id: null,
    code: `MOD${timestamp.toString().slice(-3)}`,
    year: currentYear,
    description: 'Valid code test',
    credits: 16,
  };

  beforeEach(() => {
    cy.session('admin-api', () => {
      cy.apiLoginAs('admin');
    });
  });

  it("can create module", () => {
    cy.createModule({
      code: mod.code,
      year: mod.year,
      description: mod.description,
      credits: mod.credits,
      expectedStatus: 201,
    }).then(({ moduleId }) => {
      mod.id = moduleId;
    });
  });

  it("can edit module", () => {
    const futureYear = currentYear + 1;

    // update local module state
    mod.year = futureYear;
    mod.description = 'Updated';

    cy.editModule({
      moduleId: mod.id,
      search: mod.code,
      code: mod.code,
      year: mod.year,
      description: mod.description,
      credits: mod.credits,
      expectedStatus: 200,
    });
  });

  it("should reject editing module with year before current", () => {
    const pastYear = currentYear - 1;

    cy.editModule({
      moduleId: mod.id,
      search: mod.code,
      code: mod.code,
      year: pastYear,
      description: 'Past year update',
      credits: mod.credits,
      expectedStatus: 400,
    });

    cy.contains("Year must be current year or later").should('exist');
  });

  it("can delete module", () => {
    cy.deleteModule({
      moduleId: mod.id,
      search: mod.code,
      expectedStatus: 200,
    });
  });

  it("should reject invalid module code format", () => {
    const invalidModule = {
      code: 'XYZ12',
      year: currentYear,
      description: 'Invalid code test',
      credits: 16,
    };

    cy.createModule({
      ...invalidModule,
      expectedStatus: 400,
    });

    cy.get('[data-cy="create-modal-cancel"]').click();
  });
});
