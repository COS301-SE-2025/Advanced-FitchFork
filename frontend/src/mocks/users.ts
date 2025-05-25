// src/mocks/users.ts

import type { User } from "@/types/users";


export const mockUsers: User[] = Array.from({ length: 100 }, (_, i) => {
  const id = i + 1;
  const student_number = `u${10000000 + i}`; // deterministic unique student numbers

  const firstNames = [
    'Alice', 'Bob', 'Carla', 'Daniel', 'Elena', 'Farhan', 'George', 'Hana', 'Ian', 'Julia',
    'Kabelo', 'Linda', 'Michael', 'Nadine', 'Owen', 'Paula', 'Quinton', 'Rita', 'Samuel', 'Thandi',
    'Ursula', 'Victor', 'Wendy', 'Xavier', 'Yasmin', 'Zane', 'Aaron', 'Bianca', 'Charles', 'Diane',
    'Ethan', 'Fiona', 'Gareth', 'Harper', 'Isaac', 'Jade', 'Kyle', 'Liam', 'Megan', 'Nathan',
    'Olivia', 'Peter', 'Queenie', 'Ryan', 'Sofia', 'Tristan', 'Uma', 'Vuyo', 'Willow', 'Xolani'
  ];
  const lastNames = [
    'Johnson', 'Smith', 'Mendes', 'Cho', 'Frost', 'Khan', 'Wells', 'Lee', 'Nguyen', 'Roberts',
    'Mokoena', 'Garcia', 'Tanaka', 'Ferreira', 'Jacobs', 'Moyo', 'Naidoo', 'Mensah', 'Banda', 'Mazibuko',
    'Nkosi', 'van Dyk', 'Botha', 'Kim', 'Singh', 'Lopez', 'Turner', 'Pillay', 'Sharma', 'White',
    'Martin', 'Clark', 'Adams', 'Evans', 'Green', 'Bailey', 'Moore', 'Young', 'Baker', 'Cooper',
    'King', 'Morgan', 'Ward', 'Hughes', 'Barnes', 'Pearson', 'Reed', 'Fox', 'Dean', 'Russell'
  ];

  const firstName = firstNames[i % firstNames.length];
  const lastName = lastNames[i % lastNames.length];
  const email = `${firstName.toLowerCase()}.${lastName.toLowerCase()}@up.ac.za`;

  return {
    id,
    student_number,
    email,
    is_admin: id % 7 === 0 || id === 1,
    module_roles: [] // Ensures shape consistency
  };
});
