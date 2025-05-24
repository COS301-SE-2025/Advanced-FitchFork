// src/mocks/modules.ts

import type { Module } from "@/types/modules";


const moduleCodes = [
  'COS301', 'COS332', 'COS344', 'COS110', 'COS122', 'COS226',
  'COS314', 'COS212', 'COS216', 'COS284', 'COS710', 'COS720',
  'COS750', 'COS760', 'COS790', 'COS730', 'COS330', 'COS353',
  'COS361', 'COS368', 'COS381', 'COS382', 'COS383', 'COS384'
];

const descriptions = [
  'Software Engineering',
  'Networks and Protocols',
  'Computer Graphics',
  'Introduction to Programming',
  'Discrete Structures',
  'Data Structures and Algorithms',
  'Artificial Intelligence',
  'Theory of Computation',
  'Computer Organization',
  'Databases',
  'Research Methods',
  'Advanced Algorithms',
  'High-Performance Computing',
  'Advanced Programming',
  'Project Management',
  'Compilers',
  'Operating Systems',
  'Security and Privacy',
  'Parallel Programming',
  'Cloud Computing',
  'Internet of Things',
  'Machine Learning',
  'Data Science',
  'UX and Design'
];

export const mockModules: Module[] = Array.from({ length: 30 }, (_, i) => {
  const id = i + 1;
  const code = moduleCodes[i % moduleCodes.length];
  const year = 2020 + (i % 6); // Years from 2020 to 2025
  const description = descriptions[i % descriptions.length];

  return {
    id,
    code,
    year,
    description,
  };
});
