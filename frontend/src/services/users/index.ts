import { useMock } from '../shouldUseMock';
import * as Real from './real';
import * as Mock from './mock';

export const UsersService = useMock ? Mock.UsersService : Real.UsersService;
