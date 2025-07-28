import type { 
  GetPersonnelQuery, 
  GetPersonnelResponse, 
  GetEligibleUsersQuery, 
  GetEligibleUsersResponse 
} from "@/types/modules/personnel";
import { apiFetch } from "@/utils/api";
import { buildQuery } from "@/utils/api";


export const getPersonnel = async (
  moduleId: number,
  query: GetPersonnelQuery
): Promise<GetPersonnelResponse> => {
  const q = buildQuery(query);
  return apiFetch(`/modules/${moduleId}/personnel?${q}`);
};

export const getEligibleUsers = async (
  moduleId: number,
  query: GetEligibleUsersQuery
): Promise<GetEligibleUsersResponse> => {
  const q = buildQuery(query);
  return apiFetch(`/modules/${moduleId}/eligible-users?${q}`);
};
