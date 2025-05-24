import { useMock } from '../shouldUseMock';
import * as Real from './real';
import * as Mock from './mock';

export const AuthService = useMock ? Mock.AuthService : Real.AuthService;
