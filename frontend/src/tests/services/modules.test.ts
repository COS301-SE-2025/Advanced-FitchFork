import { ModulesService } from "@/services/modules";
import { apiFetch } from "@/utils/api";

jest.mock("@/utils/api");

const mockApiFetch = apiFetch as jest.Mock;

describe("ModulesService", () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe("listModules", () => {
    it("constructs correct query string with all filters and sort", async () => {
      mockApiFetch.mockResolvedValue({ success: true, data: [], message: "ok" });

      await ModulesService.listModules({
        page: 1,
        per_page: 20,
        sort: [{ field: "code", order: "asc" }, { field: "year", order: "desc" }],
        query: "COS",
        code: "COS301",
        year: 2025
      });

      expect(mockApiFetch).toHaveBeenCalledWith(
        "/modules?page=1&per_page=20&sort=code%2C-year&query=COS&code=COS301&year=2025",
        { method: "GET" }
      );
    });
  });

  describe("getModuleDetails", () => {
    it("calls correct endpoint for module details", async () => {
      mockApiFetch.mockResolvedValue({ success: true, data: {}, message: "ok" });

      await ModulesService.getModuleDetails(123);

      expect(mockApiFetch).toHaveBeenCalledWith(
        "/modules/123",
        { method: "GET" }
      );
    });
  });

  describe("createModule", () => {
    it("sends correct POST body to create module", async () => {
      const payload = {
        code: "COS302",
        year: 2025,
        description: "Test module",
        credits: 16
      };

      mockApiFetch.mockResolvedValue({ success: true, data: {}, message: "ok" });

      await ModulesService.createModule(payload);

      expect(mockApiFetch).toHaveBeenCalledWith(
        "/modules",
        {
          method: "POST",
          body: JSON.stringify(payload)
        }
      );
    });
  });

  describe("editModule", () => {
    it("sends correct PUT body to edit module", async () => {
      const payload = {
        code: "COS303",
        year: 2025,
        description: "Updated module",
        credits: 32
      };

      mockApiFetch.mockResolvedValue({ success: true, data: {}, message: "ok" });

      await ModulesService.editModule(1, payload);

      expect(mockApiFetch).toHaveBeenCalledWith(
        "/modules/1",
        {
          method: "PUT",
          body: JSON.stringify(payload)
        }
      );
    });
  });

  describe("deleteModule", () => {
    it("sends DELETE request to remove module", async () => {
      mockApiFetch.mockResolvedValue({ success: true, data: null, message: "Deleted" });

      await ModulesService.deleteModule(1);

      expect(mockApiFetch).toHaveBeenCalledWith(
        "/modules/1",
        { method: "DELETE" }
      );
    });
  });

  describe("getMyModules", () => {
    it("fetches current user's modules", async () => {
      mockApiFetch.mockResolvedValue({ success: true, data: {}, message: "ok" });

      await ModulesService.getMyModules();

      expect(mockApiFetch).toHaveBeenCalledWith(
        "/modules/my",
        { method: "GET" }
      );
    });
  });
});
