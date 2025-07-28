import type {
  CreateUserPayload,
  BulkCreateUserPayload,
  PostUserResponse,
  PostUsersBulkResponse,
} from "@/types/users";
import { apiFetch } from "@/utils/api";

// Create a single non-admin user
export const createUser = async (
  payload: CreateUserPayload
): Promise<PostUserResponse> => {
  return apiFetch("/users", {
    method: "POST",
    body: JSON.stringify(payload),
  });
};

// Create multiple non-admin users
export const createUsersBulk = async (
  payload: BulkCreateUserPayload
): Promise<PostUsersBulkResponse> => {
  return apiFetch("/users/bulk", {
    method: "POST",
    body: JSON.stringify(payload),
  });
};
