
import { useMock } from '../shouldUseMock';
import * as Real from './real';
import * as Mock from './mock';

export const ModulesService = useMock ? Mock.ModulesService : Real.ModulesService;
