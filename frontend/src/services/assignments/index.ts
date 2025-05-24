import { useMock } from '../shouldUseMock';
import * as Real from './real';
import * as Mock from './mock';

export const AssignmentsService = useMock ? Mock.AssignmentsService : Real.AssignmentsService;
