import type {
  AssignmentDetailsResponse,
  AssignmentFile,
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

const now = new Date().toISOString();
const delay = (ms = 200 + Math.random() * 400) => new Promise((res) => setTimeout(res, ms));

let assignmentIdCounter = 200;
let fileIdCounter = 200;

const assignmentNames = [
  "Intro to Networks",
  "Socket Lab 1",
  "OSI Layer Analysis",
  "HTTP Project",
  "Group Chat Protocol",
  "UDP Benchmarking",
  "Routing Theory",
  "Wireshark Practical",
  "LAN Setup",
  "Network Security Basics",
];

const assignmentTypes = ["Assignment", "Practical"] as const;

const generateMockAssignments = (count = 20): AssignmentDetailsResponse[] => {
  return Array.from({ length: count }).map((_, i) => {
    const id = 100 + i;
    const name = assignmentNames[i % assignmentNames.length] + ` ${Math.ceil(i / 5)}`;
    const type = assignmentTypes[i % assignmentTypes.length];
    const availableFrom = new Date(Date.now() - i * 86400000 * 2).toISOString();
    const dueDate = new Date(Date.now() + (5 - i % 10) * 86400000).toISOString();

    return {
      id,
      module_id: 101,
      name,
      description: `${name} description.`,
      assignment_type: type,
      available_from: availableFrom,
      due_date: dueDate,
      created_at: now,
      updated_at: now,
      files: [],
    };
  });
};

const generateMockFiles = (assignments: AssignmentDetailsResponse[]): AssignmentFile[] => {
  const files: AssignmentFile[] = [];

  for (const a of assignments) {
    const roll = Math.random();
    const numFiles = roll < 0.6 ? 0 : roll < 0.9 ? 1 : 2; // ~60% no files, 30% one file, 10% two files

    for (let i = 0; i < numFiles; i++) {
      files.push({
        id: `f${fileIdCounter++}`,
        assignment_id: a.id,
        filename: `${a.name.toLowerCase().replace(/\s+/g, '_')}_file${i + 1}.pdf`,
        path: `/uploads/${a.name.toLowerCase().replace(/\s+/g, '_')}_file${i + 1}.pdf`,
        created_at: now,
        updated_at: now,
      });
    }
  }

  return files;
};


let mockAssignments: AssignmentDetailsResponse[] = generateMockAssignments(30);
let mockFiles: AssignmentFile[] = generateMockFiles(mockAssignments);


export const AssignmentsService = {
  listAssignments: async (
    _moduleId: number,
    _request: ListAssignmentsRequest
  ): Promise<ApiResponse<ListAssignmentsResponse>> => {
    await delay();
    return {
      success: true,
      data: {
        assignments: mockAssignments,
        page: 1,
        per_page: mockAssignments.length,
        total: mockAssignments.length,
      },
      message: "Mocked assignment list (unfiltered)",
    };
  },

  getAssignmentDetails: async (
    _moduleId: number,
    _assignmentId: number
  ): Promise<ApiResponse<AssignmentDetailsResponse>> => {
    await delay();
    return {
      success: true,
      data: {
        ...mockAssignments[0],
        files: [...mockFiles],
      },
      message: "Mocked assignment details (first assignment)",
    };
  },

  createAssignment: async (
    _moduleId: number,
    payload: CreateAssignmentRequest
  ): Promise<ApiResponse<CreateAssignmentResponse>> => {
    await delay();
    const newAssignment: AssignmentDetailsResponse = {
      id: assignmentIdCounter++,
      module_id: 999,
      name: payload.name,
      description: payload.description,
      assignment_type: payload.assignment_type,
      available_from: payload.available_from,
      due_date: payload.due_date,
      created_at: now,
      updated_at: now,
      files: [],
    };
    mockAssignments.push(newAssignment);
    return {
      success: true,
      data: newAssignment,
      message: "Mocked assignment created",
    };
  },

  editAssignment: async (
    _moduleId: number,
    _assignmentId: number,
    payload: EditAssignmentRequest
  ): Promise<ApiResponse<EditAssignmentResponse>> => {
    await delay();
    const updated = {
      ...mockAssignments[0],
      ...payload,
      updated_at: now,
    };
    mockAssignments[0] = updated;
    return {
      success: true,
      data: updated,
      message: "Mocked assignment edited (first assignment only)",
    };
  },

  deleteAssignment: async (
    _moduleId: number,
    _assignmentId: number
  ): Promise<ApiResponse<DeleteAssignmentResponse>> => {
    await delay();
    mockAssignments.pop();
    return {
      success: true,
      data: null,
      message: "Mocked assignment deleted (last one removed)",
    };
  },

  uploadFiles: async (
    _moduleId: number,
    _assignmentId: number,
    fileBlobs: File[]
  ): Promise<ApiResponse<UploadAssignmentFilesResponse>> => {
    await delay();
    const newFiles: AssignmentFile[] = fileBlobs.map(file => ({
      id: `f${fileIdCounter++}`,
      assignment_id: 999,
      filename: file.name,
      path: `/uploads/${file.name}`,
      created_at: now,
      updated_at: now,
    }));
    mockFiles.push(...newFiles);
    return {
      success: true,
      data: newFiles,
      message: "Mocked files uploaded",
    };
  },

  listFiles: async (
    _moduleId: number,
    _assignmentId: number
  ): Promise<ApiResponse<ListAssignmentFilesResponse>> => {
    await delay();
    return {
      success: true,
      data: [...mockFiles],
      message: "Mocked all files",
    };
  },

  deleteFiles: async (
    _moduleId: number,
    _assignmentId: number,
    request: DeleteAssignmentFilesRequest
  ): Promise<ApiResponse<DeleteAssignmentFilesResponse | PartialDeleteAssignmentFilesResponse>> => {
    await delay();
    const existingIds = new Set(mockFiles.map(f => f.id));
    const notFound = request.file_ids.filter(id => !existingIds.has(id));
    mockFiles = mockFiles.filter(f => !request.file_ids.includes(f.id));

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
    _moduleId: number,
    _assignmentId: number,
    _fileId: string
  ): Promise<Response> => {
    await delay();
    return new Response("Mock file content", {
      headers: { "Content-Type": "text/plain" },
      status: 200,
    });
  },
};
