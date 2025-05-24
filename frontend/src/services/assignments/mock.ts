import type {
  AssignmentDetailsResponse,
  CreateAssignmentRequest,
  CreateAssignmentResponse,
  DeleteAssignmentFilesRequest,
  DeleteAssignmentFilesResponse,
  DeleteAssignmentResponse,
  EditAssignmentRequest,
  EditAssignmentResponse,
  ListAssignmentsRequest,
  ListAssignmentsResponse,
  PartialDeleteAssignmentFilesResponse,
  UploadAssignmentFilesResponse,
  ListAssignmentFilesResponse,
} from "@/types/assignments";
import type { ApiResponse } from "@/utils/api";

const now = new Date();
let assignmentIdCounter = 1;
let fileIdCounter = 1;

const assignments: Record<string, any[]> = {}; // moduleId → assignments[]
const files: Record<string, any[]> = {}; // assignmentId → files[]

function makeAssignment(moduleId: string, payload: CreateAssignmentRequest, id?: string) {
  const newId = id ?? (assignmentIdCounter++).toString();
  return {
    id: newId,
    module_id: moduleId,
    ...payload,
    created_at: now.toISOString(),
    updated_at: now.toISOString(),
  };
}

export const AssignmentsService = {
  createAssignment: async (
    moduleId: string,
    payload: CreateAssignmentRequest
  ): Promise<ApiResponse<CreateAssignmentResponse>> => {
    const newAssignment = makeAssignment(moduleId, payload);
    assignments[moduleId] = assignments[moduleId] || [];
    assignments[moduleId].push(newAssignment);
    return {
      success: true,
      data: newAssignment,
      message: "Mocked assignment created",
    };
  },

  editAssignment: async (
    moduleId: string,
    assignmentId: string,
    payload: EditAssignmentRequest
  ): Promise<ApiResponse<EditAssignmentResponse>> => {
    const list = assignments[moduleId] || [];
    const idx = list.findIndex((a) => a.id === assignmentId);
    if (idx !== -1) {
      list[idx] = {
        ...list[idx],
        ...payload,
        updated_at: new Date().toISOString(),
      };
    }
    return {
      success: true,
      data: list[idx],
      message: "Mocked assignment edited",
    };
  },

  deleteAssignment: async (
    moduleId: string,
    assignmentId: string
  ): Promise<ApiResponse<DeleteAssignmentResponse>> => {
    assignments[moduleId] = (assignments[moduleId] || []).filter((a) => a.id !== assignmentId);
    return {
      success: true,
      data: null,
      message: "Mocked assignment deleted",
    };
  },

  getAssignmentDetails: async (
    moduleId: string,
    assignmentId: string
  ): Promise<ApiResponse<AssignmentDetailsResponse>> => {
    const assignment = (assignments[moduleId] || []).find((a) => a.id === assignmentId);
    const fileList = files[assignmentId] || [];
    return {
      success: true,
      data: {
        ...assignment,
        files: fileList,
      },
      message: "Mocked assignment details",
    };
  },

  listAssignments: async (
    moduleId: string,
    request: ListAssignmentsRequest
  ): Promise<ApiResponse<ListAssignmentsResponse>> => {
    let list = [...(assignments[moduleId] || [])];

    if (typeof request.name === 'string') {
    list = list.filter(a => a.name.toLowerCase().includes(request.name!.toLowerCase()));
    }

    if (typeof request.assignment_type === 'string') {
    list = list.filter(a => a.assignment_type === request.assignment_type);
    }

    if (typeof request.available_after === 'string') {
    list = list.filter(a => new Date(a.available_from) > new Date(request.available_after!));
    }

    if (typeof request.available_before === 'string') {
    list = list.filter(a => new Date(a.available_from) < new Date(request.available_before!));
    }

    if (typeof request.due_after === 'string') {
    list = list.filter(a => new Date(a.due_date) > new Date(request.due_after!));
    }

    if (typeof request.due_before === 'string') {
    list = list.filter(a => new Date(a.due_date) < new Date(request.due_before!));
    }

    if (request.sort) {
      for (const { field, order } of [...request.sort].reverse()) {
        list.sort((a, b) => {
          const aVal = a[field];
          const bVal = b[field];
          return order === "asc" ? (aVal > bVal ? 1 : -1) : (aVal < bVal ? 1 : -1);
        });
      }
    }

    const page = request.page;
    const perPage = request.per_page;
    const paged = list.slice((page - 1) * perPage, page * perPage);

    return {
      success: true,
      data: {
        assignments: paged,
        page,
        per_page: perPage,
        total: list.length,
      },
      message: "Mocked assignment list",
    };
  },

  uploadFiles: async (
    _moduleId: string,
    assignmentId: string,
    fileBlobs: File[]
  ): Promise<ApiResponse<UploadAssignmentFilesResponse>> => {
    const f = files[assignmentId] = files[assignmentId] || [];
    const uploaded = fileBlobs.map(file => ({
      id: (fileIdCounter++).toString(),
      assignment_id: assignmentId,
      filename: file.name,
      path: `/uploads/${file.name}`,
      created_at: now.toISOString(),
      updated_at: now.toISOString(),
    }));
    f.push(...uploaded);
    return {
      success: true,
      data: uploaded,
      message: "Mocked files uploaded",
    };
  },

  listFiles: async (
    _moduleId: string,
    assignmentId: string
  ): Promise<ApiResponse<ListAssignmentFilesResponse>> => ({
    success: true,
    data: files[assignmentId] || [],
    message: "Mocked file list",
  }),

  deleteFiles: async (
    _moduleId: string,
    assignmentId: string,
    payload: DeleteAssignmentFilesRequest
  ): Promise<ApiResponse<DeleteAssignmentFilesResponse | PartialDeleteAssignmentFilesResponse>> => {
    const all = files[assignmentId] || [];
    const before = all.length;
    files[assignmentId] = all.filter(f => !payload.file_ids.includes(f.id));
    const after = files[assignmentId].length;

    const notFound = payload.file_ids.filter(id => !all.some(f => f.id === id));
    if (notFound.length > 0) {
      return {
        success: true,
        data: { not_found: notFound },
        message: "Some files could not be deleted (mocked)",
      };
    }

    return {
      success: true,
      data: null,
      message: "Mocked files deleted",
    };
  },

  downloadFile: async (
    moduleId: string,
    assignmentId: string,
    fileId: string
  ): Promise<Response> => {
    return new Response(`Mock file data: module=${moduleId}, assignment=${assignmentId}, file=${fileId}`, {
      headers: { "Content-Type": "text/plain" },
      status: 200,
    });
  },
};
