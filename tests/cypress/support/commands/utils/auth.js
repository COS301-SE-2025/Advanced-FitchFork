/**
 * Retrieves the current user's JWT token from localStorage.
 * Returns null if not found.
 */
export function getAuthToken() {
  try {
    const auth = JSON.parse(window.localStorage.getItem('auth'));
    return auth?.token || null;
  } catch {
    return null;
  }
}

/**
 * Sets the JWT token and user data into localStorage in the expected format.
 *
 * @param {Object} user]
 * @param {string} user.id
 * @param {string} user.username
 * @param {string} user.email
 * @param {string} user.token
 * @param {boolean} [user.admin=false]
 * @param {string} [user.expires_at]
 */
export function setAuthToken({ id, username, email, token, admin = false, expires_at }) {
  const authPayload = {
    id,
    username,
    email,
    token,
    admin,
    expires_at,
  };

  window.localStorage.setItem("auth", JSON.stringify(authPayload));
}

/**
 * Retrieves the current authenticated user object from localStorage.
 * Returns null if not found or invalid.
 *
 * @returns {{
 *   id: number,
 *   username: string,
 *   email: string,
 *   token: string,
 *   admin: boolean,
 *   expires_at?: string
 * } | null}
 */
export function getAuthUser() {
  try {
    const auth = JSON.parse(window.localStorage.getItem('auth'));
    if (!auth || typeof auth !== 'object') return null;

    const { id, username, email, token, admin, expires_at } = auth;
    if (!token || !id || !username || !email) return null;

    return { id, username, email, token, admin, expires_at };
  } catch {
    return null;
  }
}
