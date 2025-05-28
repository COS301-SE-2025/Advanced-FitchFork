import { UsersService } from "@/services/users";
import { apiFetch } from "@/utils/api";

jest.mock("@/utils/api");

const mockApiFetch = apiFetch as jest.Mock;

describe("UsersService", () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe("listUsers", () => {
    it("constructs query params correctly with all filters", async () => {
      mockApiFetch.mockResolvedValue({ success: true, data: [], message: "ok" });

      await UsersService.listUsers({
        page: 1,
        per_page: 10,
        sort: [
          { field: "email", order: "asc" },
          { field: "student_number", order: "desc" }
        ],
        email: "user",
        student_number: "u123",
        admin: true
      });

      expect(mockApiFetch).toHaveBeenCalledWith(
        "/users?page=1&per_page=10&sort=email%2C-student_number&email=user&student_number=u123&admin=true",
        { method: "GET" }
      );
    });

    it("includes query param when searching", async () => {
      mockApiFetch.mockResolvedValue({ success: true, data: [], message: "ok" });

      await UsersService.listUsers({
        page: 2,
        per_page: 5,
        query: "u123"
      });

      expect(mockApiFetch).toHaveBeenCalledWith(
        "/users?page=2&per_page=5&query=u123",
        { method: "GET" }
      );
    });

    it("omits optional filters when not present", async () => {
      mockApiFetch.mockResolvedValue({ success: true, data: [], message: "ok" });

      await UsersService.listUsers({
        page: 1,
        per_page: 20
      });

      expect(mockApiFetch).toHaveBeenCalledWith(
        "/users?page=1&per_page=20",
        { method: "GET" }
      );
    });
  });

  describe("editUser", () => {
    it("calls apiFetch with correct PUT payload", async () => {
      const payload = {
        student_number: "u99999999",
        email: "new@example.com",
        admin: true
      };

      mockApiFetch.mockResolvedValue({ success: true, data: { ...payload, id: 1 }, message: "ok" });

      await UsersService.editUser(1, payload);

      expect(mockApiFetch).toHaveBeenCalledWith(
        "/users/1",
        {
          method: "PUT",
          body: JSON.stringify(payload)
        }
      );
    });
  });

  describe("deleteUser", () => {
    it("calls apiFetch with DELETE method", async () => {
      mockApiFetch.mockResolvedValue({ success: true, data: null, message: "User deleted" });

      await UsersService.deleteUser(1);

      expect(mockApiFetch).toHaveBeenCalledWith(
        "/users/1",
        { method: "DELETE" }
      );
    });
  });
});
