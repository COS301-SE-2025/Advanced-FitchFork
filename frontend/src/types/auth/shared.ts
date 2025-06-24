import type { User } from "@/types/users";

export interface JWTToken {
  token: string,
  expires_at: string
}

export interface AuthUser extends User, JWTToken {}