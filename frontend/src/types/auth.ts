import type { Module } from "./modules";
import type { User } from "./users";



// ─────────────────────────────────────────────────────────────
// TYPES - Core Domain Types
// ─────────────────────────────────────────────────────────────

export interface JWTToken {
  token: string,
  expires_at: string
}

export interface AuthUser extends User, JWTToken {}

// ─────────────────────────────────────────────────────────────
// REQUEST - API Request Payloads
// ─────────────────────────────────────────────────────────────

export interface RegisterRequest {
  username: string;
  email: string;
  password: string;
}

export interface LoginRequest {
  username: string;
  password: string;
}

// ─────────────────────────────────────────────────────────────
// RESPONSE - API Response Structures
// ─────────────────────────────────────────────────────────────

export interface UserModule extends Module {
  role: 'Lecturer' | 'Tutor' | 'Student';
}

export interface MeResponse extends User {
  modules: UserModule[]
}