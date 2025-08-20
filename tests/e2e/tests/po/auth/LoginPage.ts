import { expect, type Locator, type Page } from '@playwright/test';

/**
 * Page Object: Login page
 *
 * - Encapsulates common locators & actions for `/login`
 * - Keeps tests DRY and readable
 */
export class LoginPage {
  readonly page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  async submitAndWaitForDashboard(opts?: { loginUrlRe?: RegExp }) {
    const loginUrlRe = opts?.loginUrlRe ?? /\/api\/auth\/login\b/;
    const postOk = this.page.waitForResponse(r =>
      r.request().method() === 'POST' &&
      loginUrlRe.test(r.url()) &&
      r.status() >= 200 && r.status() < 300
    );

    await this.submit().click();
    await postOk;                            // server accepted credentials
    await expect(this.page).toHaveURL(/\/dashboard\b/); // now the client redirect
  }

  // ----------------- navigation -----------------
  async goto() {
    await this.page.goto('/login');
    await expect(this.page.getByRole('heading', { name: 'Login' })).toBeVisible();
  }

  // ----------------- core locators -----------------
  username(): Locator {
    return this.page.getByLabel(/username/i);
  }
  password(): Locator {
    return this.page.getByLabel(/^password$/i);
  }
  submit(): Locator {
    return this.page.getByRole('button', { name: /sign in/i });
  }
  alert(): Locator {
    return this.page.getByRole('alert');
  }
  forgotPasswordLink(): Locator {
    return this.page.getByRole('link', { name: /forgot password\?/i });
  }
  signupLink(): Locator {
    return this.page.getByRole('link', { name: /sign up/i });
  }

  // ----------------- actions -----------------
  async fillAndSubmit(data: { username: string; password: string }) {
    await this.username().fill(data.username);
    await this.password().fill(data.password);
    await this.submit().click();
  }
}
