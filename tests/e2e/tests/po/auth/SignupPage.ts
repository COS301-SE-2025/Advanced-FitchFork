import { expect, type Locator, type Page } from '@playwright/test';

/**
 * Page Object: Signup page
 *
 * - Encapsulates locators & actions for `/signup`
 * - Works with AntD form fields + alert
 */
export class SignupPage {
  readonly page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  // ----------------- navigation -----------------
  async goto() {
    await this.page.goto('/signup');
    await expect(this.page.getByRole('heading', { name: 'Signup' })).toBeVisible();
  }

  // ----------------- core locators -----------------
  username(): Locator {
    return this.page.getByLabel(/username/i);
  }
  email(): Locator {
    return this.page.getByLabel(/email/i);
  }
  password(): Locator {
    return this.page.getByLabel(/^password$/i);
  }
  confirm(): Locator {
    return this.page.getByLabel(/confirm password/i);
  }
  submit(): Locator {
    return this.page.getByRole('button', { name: /create account/i });
  }
  errorAlert(): Locator {
    return this.page.getByRole('alert'); // AntD Alert
  }

  // ----------------- actions -----------------
  async fillAndSubmit(data: { username: string; email: string; password: string; confirm: string }) {
    await this.username().fill(data.username);
    await this.email().fill(data.email);
    await this.password().fill(data.password);
    await this.confirm().fill(data.confirm);
    await this.submit().click();
  }
}
