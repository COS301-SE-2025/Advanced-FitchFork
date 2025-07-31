import http from 'k6/http';

/**
 * Sends an authorized JSON POST request
 */
export function authorizedPost(url, payload, token, isJson = true) {
  const headers = {
    Authorization: `Bearer ${token}`,
    ...(isJson ? { 'Content-Type': 'application/json' } : {})
  };
  return http.post(url, isJson ? JSON.stringify(payload) : payload, { headers });
}

/**
 * Sends an authorized multipart form-data POST request
 * Used for file uploads (e.g., config, main, memo, makefile)
 */
export function authorizedFileUpload(url, formData, token) {
  const headers = {
    Authorization: `Bearer ${token}`,
    // k6 will automatically set the correct Content-Type boundary
  };

  return http.post(url, formData, { headers });
}
