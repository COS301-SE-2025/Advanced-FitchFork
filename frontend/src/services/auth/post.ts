import type {
  PostLoginResponse,
  PostRegisterResponse,
  PostRequestPasswordResetResponse, 
  PostResetPasswordResponse, 
  PostUploadProfilePictureResponse, 
  PostVerifyResetTokenResponse} from "@/types/auth";
import { api, apiUpload } from "@/utils/api";

export const login = async (
  username: string,
  password: string,
): Promise<PostLoginResponse> => {
  return api.post('/auth/login', { username, password });
};

export const register = async (
  username: string,
  email: string,
  password: string,
): Promise<PostRegisterResponse> => {
  return api.post('/auth/register', { username, email, password });
};

export const requestPasswordReset = async (
  email: string
): Promise<PostRequestPasswordResetResponse> => {
  return api.post('/auth/request-password-reset', { email });
};

export const resetPassword = async (
  token: string,
  new_password: string
): Promise<PostResetPasswordResponse> => {
  return api.post('/auth/reset-password', {
      token,
      new_password
    });
};


export const verifyResetToken = async (
  token: string
): Promise<PostVerifyResetTokenResponse> => {
  return api.post('/auth/verify-reset-token', {
    token
  });
};

export const uploadProfilePicture = async (
  form: FormData
): Promise<PostUploadProfilePictureResponse> => {
  return apiUpload('/auth/upload-profile-picture', form);
};

export const changePassword = async (
  current_password: string,
  new_password: string,
) => {
  return api.post('/auth/change-password', { current_password, new_password })
}