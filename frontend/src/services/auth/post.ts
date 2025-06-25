import type {
  PostLoginResponse,
  PostRegisterResponse,
  PostRequestPasswordResetResponse, 
  PostResetPasswordResponse, 
  PostUploadProfilePictureResponse, 
  PostVerifyResetTokenResponse} from "@/types/auth";
import { apiFetch, apiUpload } from "@/utils/api";

export const login = async (
  username: string,
  password: string,
): Promise<PostLoginResponse> => {
  return apiFetch('/auth/login', {
    method: 'POST',
    body: JSON.stringify({ username, password }),
  });
};

export const register = async (
  username: string,
  email: string,
  password: string,
): Promise<PostRegisterResponse> => {
  return apiFetch('/auth/register', {
    method: 'POST',
    body: JSON.stringify({ username, email, password }),
  });
};

export const requestPasswordReset = async (
  email: string
): Promise<PostRequestPasswordResetResponse> => {
  return apiFetch('/auth/request-password-reset', {
    method: 'POST',
    body: JSON.stringify({ email }),
  });
};

export const resetPassword = async (
  token: string,
  newPassword: string
): Promise<PostResetPasswordResponse> => {
  return apiFetch('/auth/reset-password', {
    method: 'POST',
    body: JSON.stringify({
      token,
      new_password: newPassword,
    }),
  });
};


export const verifyResetToken = async (
  token: string
): Promise<PostVerifyResetTokenResponse> => {
  return apiFetch('/auth/verify-reset-token', {
    method: 'POST',
    body: JSON.stringify({ token }),
  });
};

export const uploadProfilePicture = async (
  form: FormData
): Promise<PostUploadProfilePictureResponse> => {
  return apiUpload('/auth/upload-profile-picture', form);
};