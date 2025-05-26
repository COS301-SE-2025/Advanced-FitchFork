import { AssignmentsService } from "@/services/assignments/real";
import { apiFetch } from "@/utils/api";

jest.mock("@/utils/api");
const mockApiFetch = apiFetch as jest.Mock;

describe("AssignmentsService", () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe("createAssignment", () => {
    it("sends correct POST request to create assignment", async () => {
      const payload = {
        name: "Assignment 1",
        description: "Design task",
        assignment_type: "Assignment" as "Assignment" | "Practical",
        available_from: "2025-06-01T08:00:00Z",
        due_date: "2025-06-15T23:59:00Z"
      };

      mockApiFetch.mockResolvedValue({ success: true, data: {}, message: "ok" });

      await AssignmentsService.createAssignment("mod-123", payload);

      expect(mockApiFetch).toHaveBeenCalledWith(
        "/modules/mod-123/assignments",
        {
          method: "POST",
          body: JSON.stringify(payload)
        }
      );
    });
  });

  describe("editAssignment", () => {
    it("sends correct PUT request to edit assignment", async () => {
      const payload = {
        name: "Updated Assignment"
      };
      mockApiFetch.mockResolvedValue({ success: true, data: {}, message: "ok" });

      await AssignmentsService.editAssignment("mod-123", "asg-456", payload);

      expect(mockApiFetch).toHaveBeenCalledWith(
        "/modules/mod-123/assignments/asg-456",
        {
          method: "PUT",
          body: JSON.stringify(payload)
        }
      );
    });
  });

  describe("listAssignments", () => {
    it("constructs correct query params with filters and sort", async () => {
      mockApiFetch.mockResolvedValue({ success: true, data: [], message: "ok" });

      await AssignmentsService.listAssignments("mod-123", {
        page: 1,
        per_page: 10,
        sort: [{ field: "due_date", order: "desc" }, { field: "name", order: "asc" }],
        name: "design",
        assignment_type: "Practical",
        available_after: "2024-07-01",
        available_before: "2024-08-01",
        due_after: "2024-08-01",
        due_before: "2024-08-31"
      });

      expect(mockApiFetch).toHaveBeenCalledWith(
        "/modules/mod-123/assignments?page=1&per_page=10&sort=-due_date%2Cname&name=design&assignment_type=Practical&available_before=2024-08-01&available_after=2024-07-01&due_before=2024-08-31&due_after=2024-08-01",
        { method: "GET" }
      );
    });
  });

  describe("deleteAssignment", () => {
    it("sends DELETE request to correct URL", async () => {
      mockApiFetch.mockResolvedValue({ success: true, data: null, message: "ok" });

      await AssignmentsService.deleteAssignment("mod-1", "asg-2");

      expect(mockApiFetch).toHaveBeenCalledWith(
        "/modules/mod-1/assignments/asg-2",
        { method: "DELETE" }
      );
    });
  });

  describe("deleteFiles", () => {
    it("sends correct DELETE body to remove files", async () => {
      const payload = { file_ids: ["f1", "f2"] };
      mockApiFetch.mockResolvedValue({ success: true, data: {}, message: "ok" });

      await AssignmentsService.deleteFiles("mod-1", "asg-2", payload);

      expect(mockApiFetch).toHaveBeenCalledWith(
        "/modules/mod-1/assignments/asg-2/files",
        {
          method: "DELETE",
          body: JSON.stringify(payload)
        }
      );
    });
  });
});
