import type { AuthUser } from ".";
import type { ApiResponse } from "@/types/common";
import type { User } from "@/types/users";
import type { Module } from "@/types/modules"

// ─────────────────────────────────────────────────────────────
// GET Response Types
// ─────────────────────────────────────────────────────────────

export type GetAuthenticatedUserResponse = ApiResponse<{ modules: Module[] } & User>;
export type GetHasRoleResponse = ApiResponse<{ has_role: boolean }>;

// ─────────────────────────────────────────────────────────────
// POST Response Types
// ─────────────────────────────────────────────────────────────

export type PostLoginResponse = ApiResponse<AuthUser | null>;
export type PostRegisterResponse = ApiResponse<AuthUser | null>;
export type PostRequestPasswordResetResponse = ApiResponse<null>;
export type PostResetPasswordResponse = ApiResponse<null>;
export type PostVerifyResetTokenResponse = ApiResponse<{ email_hint?: string }>;
export type PostUploadProfilePictureResponse = ApiResponse<{ profile_picture_path: string }>;
