import { AuthService } from "@/services/auth";
import { apiFetch } from "@/utils/api";

jest.mock("@/utils/api");

describe("AuthService", () => {
  const mockApiFetch = apiFetch as jest.Mock;

  beforeEach(() => {
    mockApiFetch.mockReset();
  });

  describe("login", () => {
    it("should call apiFetch with correct URL and payload", async () => {
      const payload = { student_number: "u12345678", password: "pass123" };
      const mockResponse = { success: true, message: "Login successful", data: { ...payload, token: "jwt", expires_at: "date", email: "test@up.ac.za", id: 1, admin: false, module_roles: [] } };
      mockApiFetch.mockResolvedValueOnce(mockResponse);

      const result = await AuthService.login(payload);

      expect(mockApiFetch).toHaveBeenCalledWith("/auth/login", {
        method: "POST",
        body: JSON.stringify(payload),
      });
      expect(result).toEqual(mockResponse);
    });
  });

  describe("register", () => {
    it("should call apiFetch with correct URL and payload", async () => {
      const payload = { student_number: "u12345678", email: "test@up.ac.za", password: "pass123" };
      const mockResponse = { success: true, message: "Register successful", data: { ...payload, token: "jwt", expires_at: "date", id: 1, admin: false, module_roles: [] } };
      mockApiFetch.mockResolvedValueOnce(mockResponse);

      const result = await AuthService.register(payload);

      expect(mockApiFetch).toHaveBeenCalledWith("/auth/register", {
        method: "POST",
        body: JSON.stringify(payload),
      });
      expect(result).toEqual(mockResponse);
    });
  });

  describe("me", () => {
    it("should call apiFetch with GET method", async () => {
      const mockResponse = {
        success: true,
        message: "User fetched",
        data: {
          id: 1,
          student_number: "u12345678",
          email: "test@up.ac.za",
          admin: true,
          module_roles: [],
          modules: [],
        },
      };
      mockApiFetch.mockResolvedValueOnce(mockResponse);

      const result = await AuthService.me();

      expect(mockApiFetch).toHaveBeenCalledWith("/auth/me", {
        method: "GET",
      });
      expect(result).toEqual(mockResponse);
    });
  });
});