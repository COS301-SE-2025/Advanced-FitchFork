# Details

Date : 2025-08-18 20:56:42

Directory c:\\Users\\rxxim\\Advanced-FitchFork

Total : 622 files,  79047 codes, 9487 comments, 9074 blanks, all 97608 lines

[Summary](results.md) / Details / [Diff Summary](diff.md) / [Diff Details](diff-details.md)

## Files
| filename | language | code | comment | blank | total |
| :--- | :--- | ---: | ---: | ---: | ---: |
| [.github/actions/start-servers/action.yml](/.github/actions/start-servers/action.yml) | YAML | 223 | 7 | 21 | 251 |
| [.github/workflows/ci.yml](/.github/workflows/ci.yml) | YAML | 138 | 8 | 25 | 171 |
| [README.md](/README.md) | Markdown | 144 | 0 | 61 | 205 |
| [backend/.cargo/config.toml](/backend/.cargo/config.toml) | TOML | 2 | 0 | 0 | 2 |
| [backend/Cargo.lock](/backend/Cargo.lock) | TOML | 4,698 | 2 | 502 | 5,202 |
| [backend/Cargo.toml](/backend/Cargo.toml) | TOML | 14 | 0 | 1 | 15 |
| [backend/INTEGRATION\_TESTS.md](/backend/INTEGRATION_TESTS.md) | Markdown | 20 | 0 | 10 | 30 |
| [backend/Makefile.toml](/backend/Makefile.toml) | TOML | 21 | 0 | 7 | 28 |
| [backend/README.md](/backend/README.md) | Markdown | 54 | 0 | 29 | 83 |
| [backend/UNIT\_TESTS.md](/backend/UNIT_TESTS.md) | Markdown | 22 | 0 | 11 | 33 |
| [backend/ai/Cargo.toml](/backend/ai/Cargo.toml) | TOML | 15 | 0 | 2 | 17 |
| [backend/ai/src/algorithms/code\_coverage.rs](/backend/ai/src/algorithms/code_coverage.rs) | Rust | 18 | 5 | 6 | 29 |
| [backend/ai/src/algorithms/genetic\_algorithm.rs](/backend/ai/src/algorithms/genetic_algorithm.rs) | Rust | 459 | 53 | 70 | 582 |
| [backend/ai/src/algorithms/mod.rs](/backend/ai/src/algorithms/mod.rs) | Rust | 3 | 0 | 1 | 4 |
| [backend/ai/src/algorithms/rng.rs](/backend/ai/src/algorithms/rng.rs) | Rust | 133 | 2 | 20 | 155 |
| [backend/ai/src/lib.rs](/backend/ai/src/lib.rs) | Rust | 252 | 151 | 55 | 458 |
| [backend/ai/src/utils/attributes.rs](/backend/ai/src/utils/attributes.rs) | Rust | 27 | 11 | 6 | 44 |
| [backend/ai/src/utils/evaluator.rs](/backend/ai/src/utils/evaluator.rs) | Rust | 582 | 50 | 76 | 708 |
| [backend/ai/src/utils/mod.rs](/backend/ai/src/utils/mod.rs) | Rust | 3 | 0 | 0 | 3 |
| [backend/ai/src/utils/output.rs](/backend/ai/src/utils/output.rs) | Rust | 103 | 11 | 20 | 134 |
| [backend/api/Cargo.toml](/backend/api/Cargo.toml) | TOML | 59 | 9 | 13 | 81 |
| [backend/api/src/auth/claims.rs](/backend/api/src/auth/claims.rs) | Rust | 9 | 0 | 2 | 11 |
| [backend/api/src/auth/extractors.rs](/backend/api/src/auth/extractors.rs) | Rust | 42 | 21 | 7 | 70 |
| [backend/api/src/auth/guards.rs](/backend/api/src/auth/guards.rs) | Rust | 515 | 15 | 65 | 595 |
| [backend/api/src/auth/middleware.rs](/backend/api/src/auth/middleware.rs) | Rust | 46 | 28 | 7 | 81 |
| [backend/api/src/auth/mod.rs](/backend/api/src/auth/mod.rs) | Rust | 29 | 4 | 8 | 41 |
| [backend/api/src/lib.rs](/backend/api/src/lib.rs) | Rust | 5 | 0 | 0 | 5 |
| [backend/api/src/main.rs](/backend/api/src/main.rs) | Rust | 73 | 5 | 18 | 96 |
| [backend/api/src/response/mod.rs](/backend/api/src/response/mod.rs) | Rust | 37 | 56 | 7 | 100 |
| [backend/api/src/routes/auth/get.rs](/backend/api/src/routes/auth/get.rs) | Rust | 233 | 137 | 29 | 399 |
| [backend/api/src/routes/auth/mod.rs](/backend/api/src/routes/auth/mod.rs) | Rust | 24 | 32 | 4 | 60 |
| [backend/api/src/routes/auth/post.rs](/backend/api/src/routes/auth/post.rs) | Rust | 495 | 322 | 70 | 887 |
| [backend/api/src/routes/common.rs](/backend/api/src/routes/common.rs) | Rust | 34 | 0 | 3 | 37 |
| [backend/api/src/routes/health.rs](/backend/api/src/routes/health.rs) | Rust | 9 | 26 | 3 | 38 |
| [backend/api/src/routes/me/announcements.rs](/backend/api/src/routes/me/announcements.rs) | Rust | 148 | 55 | 26 | 229 |
| [backend/api/src/routes/me/assignments.rs](/backend/api/src/routes/me/assignments.rs) | Rust | 140 | 54 | 23 | 217 |
| [backend/api/src/routes/me/grades.rs](/backend/api/src/routes/me/grades.rs) | Rust | 246 | 85 | 31 | 362 |
| [backend/api/src/routes/me/mod.rs](/backend/api/src/routes/me/mod.rs) | Rust | 15 | 21 | 4 | 40 |
| [backend/api/src/routes/me/submissions.rs](/backend/api/src/routes/me/submissions.rs) | Rust | 258 | 75 | 32 | 365 |
| [backend/api/src/routes/me/tickets.rs](/backend/api/src/routes/me/tickets.rs) | Rust | 217 | 66 | 32 | 315 |
| [backend/api/src/routes/mod.rs](/backend/api/src/routes/mod.rs) | Rust | 20 | 23 | 3 | 46 |
| [backend/api/src/routes/modules/announcements/common.rs](/backend/api/src/routes/modules/announcements/common.rs) | Rust | 7 | 4 | 3 | 14 |
| [backend/api/src/routes/modules/announcements/delete.rs](/backend/api/src/routes/modules/announcements/delete.rs) | Rust | 25 | 30 | 3 | 58 |
| [backend/api/src/routes/modules/announcements/get.rs](/backend/api/src/routes/modules/announcements/get.rs) | Rust | 158 | 49 | 16 | 223 |
| [backend/api/src/routes/modules/announcements/mod.rs](/backend/api/src/routes/modules/announcements/mod.rs) | Rust | 23 | 23 | 5 | 51 |
| [backend/api/src/routes/modules/announcements/post.rs](/backend/api/src/routes/modules/announcements/post.rs) | Rust | 28 | 39 | 4 | 71 |
| [backend/api/src/routes/modules/announcements/put.rs](/backend/api/src/routes/modules/announcements/put.rs) | Rust | 24 | 40 | 4 | 68 |
| [backend/api/src/routes/modules/assignments/common.rs](/backend/api/src/routes/modules/assignments/common.rs) | Rust | 69 | 11 | 10 | 90 |
| [backend/api/src/routes/modules/assignments/config/get.rs](/backend/api/src/routes/modules/assignments/config/get.rs) | Rust | 101 | 87 | 7 | 195 |
| [backend/api/src/routes/modules/assignments/config/mod.rs](/backend/api/src/routes/modules/assignments/config/mod.rs) | Rust | 15 | 12 | 3 | 30 |
| [backend/api/src/routes/modules/assignments/config/post.rs](/backend/api/src/routes/modules/assignments/config/post.rs) | Rust | 86 | 48 | 8 | 142 |
| [backend/api/src/routes/modules/assignments/delete.rs](/backend/api/src/routes/modules/assignments/delete.rs) | Rust | 74 | 75 | 11 | 160 |
| [backend/api/src/routes/modules/assignments/files/delete.rs](/backend/api/src/routes/modules/assignments/files/delete.rs) | Rust | 82 | 45 | 8 | 135 |
| [backend/api/src/routes/modules/assignments/files/get.rs](/backend/api/src/routes/modules/assignments/files/get.rs) | Rust | 111 | 71 | 11 | 193 |
| [backend/api/src/routes/modules/assignments/files/mod.rs](/backend/api/src/routes/modules/assignments/files/mod.rs) | Rust | 16 | 18 | 3 | 37 |
| [backend/api/src/routes/modules/assignments/files/post.rs](/backend/api/src/routes/modules/assignments/files/post.rs) | Rust | 151 | 54 | 10 | 215 |
| [backend/api/src/routes/modules/assignments/get.rs](/backend/api/src/routes/modules/assignments/get.rs) | Rust | 507 | 238 | 51 | 796 |
| [backend/api/src/routes/modules/assignments/interpreter/delete.rs](/backend/api/src/routes/modules/assignments/interpreter/delete.rs) | Rust | 53 | 35 | 6 | 94 |
| [backend/api/src/routes/modules/assignments/interpreter/get.rs](/backend/api/src/routes/modules/assignments/interpreter/get.rs) | Rust | 85 | 14 | 9 | 108 |
| [backend/api/src/routes/modules/assignments/interpreter/mod.rs](/backend/api/src/routes/modules/assignments/interpreter/mod.rs) | Rust | 33 | 18 | 4 | 55 |
| [backend/api/src/routes/modules/assignments/interpreter/post.rs](/backend/api/src/routes/modules/assignments/interpreter/post.rs) | Rust | 132 | 18 | 10 | 160 |
| [backend/api/src/routes/modules/assignments/mark\_allocator/get.rs](/backend/api/src/routes/modules/assignments/mark_allocator/get.rs) | Rust | 28 | 92 | 3 | 123 |
| [backend/api/src/routes/modules/assignments/mark\_allocator/mod.rs](/backend/api/src/routes/modules/assignments/mark_allocator/mod.rs) | Rust | 14 | 13 | 3 | 30 |
| [backend/api/src/routes/modules/assignments/mark\_allocator/post.rs](/backend/api/src/routes/modules/assignments/mark_allocator/post.rs) | Rust | 29 | 72 | 4 | 105 |
| [backend/api/src/routes/modules/assignments/mark\_allocator/put.rs](/backend/api/src/routes/modules/assignments/mark_allocator/put.rs) | Rust | 81 | 127 | 7 | 215 |
| [backend/api/src/routes/modules/assignments/memo\_output/get.rs](/backend/api/src/routes/modules/assignments/memo_output/get.rs) | Rust | 95 | 34 | 15 | 144 |
| [backend/api/src/routes/modules/assignments/memo\_output/mod.rs](/backend/api/src/routes/modules/assignments/memo_output/mod.rs) | Rust | 11 | 6 | 2 | 19 |
| [backend/api/src/routes/modules/assignments/memo\_output/post.rs](/backend/api/src/routes/modules/assignments/memo_output/post.rs) | Rust | 79 | 85 | 8 | 172 |
| [backend/api/src/routes/modules/assignments/mod.rs](/backend/api/src/routes/modules/assignments/mod.rs) | Rust | 185 | 38 | 4 | 227 |
| [backend/api/src/routes/modules/assignments/plagiarism/delete.rs](/backend/api/src/routes/modules/assignments/plagiarism/delete.rs) | Rust | 129 | 125 | 11 | 265 |
| [backend/api/src/routes/modules/assignments/plagiarism/get.rs](/backend/api/src/routes/modules/assignments/plagiarism/get.rs) | Rust | 310 | 104 | 46 | 460 |
| [backend/api/src/routes/modules/assignments/plagiarism/mod.rs](/backend/api/src/routes/modules/assignments/plagiarism/mod.rs) | Rust | 25 | 24 | 4 | 53 |
| [backend/api/src/routes/modules/assignments/plagiarism/patch.rs](/backend/api/src/routes/modules/assignments/plagiarism/patch.rs) | Rust | 116 | 116 | 12 | 244 |
| [backend/api/src/routes/modules/assignments/plagiarism/post.rs](/backend/api/src/routes/modules/assignments/plagiarism/post.rs) | Rust | 182 | 132 | 19 | 333 |
| [backend/api/src/routes/modules/assignments/plagiarism/put.rs](/backend/api/src/routes/modules/assignments/plagiarism/put.rs) | Rust | 118 | 89 | 11 | 218 |
| [backend/api/src/routes/modules/assignments/post.rs](/backend/api/src/routes/modules/assignments/post.rs) | Rust | 100 | 64 | 6 | 170 |
| [backend/api/src/routes/modules/assignments/put.rs](/backend/api/src/routes/modules/assignments/put.rs) | Rust | 253 | 138 | 27 | 418 |
| [backend/api/src/routes/modules/assignments/submissions/common.rs](/backend/api/src/routes/modules/assignments/submissions/common.rs) | Rust | 83 | 25 | 12 | 120 |
| [backend/api/src/routes/modules/assignments/submissions/get.rs](/backend/api/src/routes/modules/assignments/submissions/get.rs) | Rust | 604 | 212 | 66 | 882 |
| [backend/api/src/routes/modules/assignments/submissions/mod.rs](/backend/api/src/routes/modules/assignments/submissions/mod.rs) | Rust | 45 | 19 | 3 | 67 |
| [backend/api/src/routes/modules/assignments/submissions/post.rs](/backend/api/src/routes/modules/assignments/submissions/post.rs) | Rust | 766 | 254 | 92 | 1,112 |
| [backend/api/src/routes/modules/assignments/tasks/common.rs](/backend/api/src/routes/modules/assignments/tasks/common.rs) | Rust | 10 | 0 | 1 | 11 |
| [backend/api/src/routes/modules/assignments/tasks/delete.rs](/backend/api/src/routes/modules/assignments/tasks/delete.rs) | Rust | 33 | 35 | 2 | 70 |
| [backend/api/src/routes/modules/assignments/tasks/get.rs](/backend/api/src/routes/modules/assignments/tasks/get.rs) | Rust | 192 | 120 | 23 | 335 |
| [backend/api/src/routes/modules/assignments/tasks/mod.rs](/backend/api/src/routes/modules/assignments/tasks/mod.rs) | Rust | 19 | 17 | 3 | 39 |
| [backend/api/src/routes/modules/assignments/tasks/post.rs](/backend/api/src/routes/modules/assignments/tasks/post.rs) | Rust | 75 | 109 | 8 | 192 |
| [backend/api/src/routes/modules/assignments/tasks/put.rs](/backend/api/src/routes/modules/assignments/tasks/put.rs) | Rust | 45 | 142 | 7 | 194 |
| [backend/api/src/routes/modules/assignments/tickets/common.rs](/backend/api/src/routes/modules/assignments/tickets/common.rs) | Rust | 50 | 25 | 8 | 83 |
| [backend/api/src/routes/modules/assignments/tickets/delete.rs](/backend/api/src/routes/modules/assignments/tickets/delete.rs) | Rust | 37 | 42 | 6 | 85 |
| [backend/api/src/routes/modules/assignments/tickets/get.rs](/backend/api/src/routes/modules/assignments/tickets/get.rs) | Rust | 215 | 106 | 22 | 343 |
| [backend/api/src/routes/modules/assignments/tickets/mod.rs](/backend/api/src/routes/modules/assignments/tickets/mod.rs) | Rust | 23 | 22 | 3 | 48 |
| [backend/api/src/routes/modules/assignments/tickets/post.rs](/backend/api/src/routes/modules/assignments/tickets/post.rs) | Rust | 43 | 49 | 6 | 98 |
| [backend/api/src/routes/modules/assignments/tickets/put.rs](/backend/api/src/routes/modules/assignments/tickets/put.rs) | Rust | 87 | 80 | 13 | 180 |
| [backend/api/src/routes/modules/assignments/tickets/ticket\_messages/common.rs](/backend/api/src/routes/modules/assignments/tickets/ticket_messages/common.rs) | Rust | 30 | 17 | 5 | 52 |
| [backend/api/src/routes/modules/assignments/tickets/ticket\_messages/delete.rs](/backend/api/src/routes/modules/assignments/tickets/ticket_messages/delete.rs) | Rust | 41 | 42 | 8 | 91 |
| [backend/api/src/routes/modules/assignments/tickets/ticket\_messages/get.rs](/backend/api/src/routes/modules/assignments/tickets/ticket_messages/get.rs) | Rust | 123 | 51 | 13 | 187 |
| [backend/api/src/routes/modules/assignments/tickets/ticket\_messages/mod.rs](/backend/api/src/routes/modules/assignments/tickets/ticket_messages/mod.rs) | Rust | 18 | 17 | 5 | 40 |
| [backend/api/src/routes/modules/assignments/tickets/ticket\_messages/post.rs](/backend/api/src/routes/modules/assignments/tickets/ticket_messages/post.rs) | Rust | 90 | 70 | 11 | 171 |
| [backend/api/src/routes/modules/assignments/tickets/ticket\_messages/put.rs](/backend/api/src/routes/modules/assignments/tickets/ticket_messages/put.rs) | Rust | 66 | 64 | 9 | 139 |
| [backend/api/src/routes/modules/common.rs](/backend/api/src/routes/modules/common.rs) | Rust | 89 | 8 | 14 | 111 |
| [backend/api/src/routes/modules/delete.rs](/backend/api/src/routes/modules/delete.rs) | Rust | 118 | 77 | 13 | 208 |
| [backend/api/src/routes/modules/get.rs](/backend/api/src/routes/modules/get.rs) | Rust | 359 | 210 | 36 | 605 |
| [backend/api/src/routes/modules/mod.rs](/backend/api/src/routes/modules/mod.rs) | Rust | 30 | 26 | 3 | 59 |
| [backend/api/src/routes/modules/personnel/delete.rs](/backend/api/src/routes/modules/personnel/delete.rs) | Rust | 109 | 87 | 15 | 211 |
| [backend/api/src/routes/modules/personnel/get.rs](/backend/api/src/routes/modules/personnel/get.rs) | Rust | 220 | 102 | 28 | 350 |
| [backend/api/src/routes/modules/personnel/mod.rs](/backend/api/src/routes/modules/personnel/mod.rs) | Rust | 12 | 19 | 4 | 35 |
| [backend/api/src/routes/modules/personnel/post.rs](/backend/api/src/routes/modules/personnel/post.rs) | Rust | 124 | 81 | 16 | 221 |
| [backend/api/src/routes/modules/post.rs](/backend/api/src/routes/modules/post.rs) | Rust | 71 | 74 | 6 | 151 |
| [backend/api/src/routes/modules/put.rs](/backend/api/src/routes/modules/put.rs) | Rust | 179 | 129 | 28 | 336 |
| [backend/api/src/routes/users/common.rs](/backend/api/src/routes/users/common.rs) | Rust | 38 | 0 | 7 | 45 |
| [backend/api/src/routes/users/delete.rs](/backend/api/src/routes/users/delete.rs) | Rust | 36 | 49 | 3 | 88 |
| [backend/api/src/routes/users/get.rs](/backend/api/src/routes/users/get.rs) | Rust | 204 | 120 | 23 | 347 |
| [backend/api/src/routes/users/mod.rs](/backend/api/src/routes/users/mod.rs) | Rust | 23 | 27 | 4 | 54 |
| [backend/api/src/routes/users/post.rs](/backend/api/src/routes/users/post.rs) | Rust | 85 | 45 | 12 | 142 |
| [backend/api/src/routes/users/put.rs](/backend/api/src/routes/users/put.rs) | Rust | 207 | 121 | 33 | 361 |
| [backend/api/src/services/email.rs](/backend/api/src/services/email.rs) | Rust | 149 | 48 | 11 | 208 |
| [backend/api/src/services/mod.rs](/backend/api/src/services/mod.rs) | Rust | 2 | 3 | 1 | 6 |
| [backend/api/src/services/moss.rs](/backend/api/src/services/moss.rs) | Rust | 195 | 17 | 40 | 252 |
| [backend/api/src/ws/handlers.rs](/backend/api/src/ws/handlers.rs) | Rust | 129 | 39 | 10 | 178 |
| [backend/api/src/ws/mod.rs](/backend/api/src/ws/mod.rs) | Rust | 12 | 18 | 5 | 35 |
| [backend/api/src/ws/modules/assignments/mod.rs](/backend/api/src/ws/modules/assignments/mod.rs) | Rust | 7 | 8 | 3 | 18 |
| [backend/api/src/ws/modules/mod.rs](/backend/api/src/ws/modules/mod.rs) | Rust | 11 | 12 | 5 | 28 |
| [backend/api/tests/auth/extractors\_test.rs](/backend/api/tests/auth/extractors_test.rs) | Rust | 116 | 0 | 22 | 138 |
| [backend/api/tests/auth/guards\_test.rs](/backend/api/tests/auth/guards_test.rs) | Rust | 636 | 2 | 87 | 725 |
| [backend/api/tests/auth/mod.rs](/backend/api/tests/auth/mod.rs) | Rust | 2 | 0 | 0 | 2 |
| [backend/api/tests/helpers/app.rs](/backend/api/tests/helpers/app.rs) | Rust | 32 | 0 | 4 | 36 |
| [backend/api/tests/helpers/mod.rs](/backend/api/tests/helpers/mod.rs) | Rust | 4 | 0 | 2 | 6 |
| [backend/api/tests/helpers/ws.rs](/backend/api/tests/helpers/ws.rs) | Rust | 47 | 2 | 6 | 55 |
| [backend/api/tests/mod.rs](/backend/api/tests/mod.rs) | Rust | 4 | 0 | 0 | 4 |
| [backend/api/tests/routes/auth/get\_test.rs](/backend/api/tests/routes/auth/get_test.rs) | Rust | 416 | 20 | 85 | 521 |
| [backend/api/tests/routes/auth/mod.rs](/backend/api/tests/routes/auth/mod.rs) | Rust | 2 | 0 | 0 | 2 |
| [backend/api/tests/routes/auth/post\_test.rs](/backend/api/tests/routes/auth/post_test.rs) | Rust | 587 | 23 | 126 | 736 |
| [backend/api/tests/routes/health\_test.rs](/backend/api/tests/routes/health_test.rs) | Rust | 26 | 0 | 4 | 30 |
| [backend/api/tests/routes/me/grades\_test.rs](/backend/api/tests/routes/me/grades_test.rs) | Rust | 309 | 0 | 46 | 355 |
| [backend/api/tests/routes/me/mod.rs](/backend/api/tests/routes/me/mod.rs) | Rust | 2 | 0 | 0 | 2 |
| [backend/api/tests/routes/me/submissions\_test.rs](/backend/api/tests/routes/me/submissions_test.rs) | Rust | 193 | 2 | 22 | 217 |
| [backend/api/tests/routes/mod.rs](/backend/api/tests/routes/mod.rs) | Rust | 5 | 0 | 0 | 5 |
| [backend/api/tests/routes/modules/announcements/delete\_test.rs](/backend/api/tests/routes/modules/announcements/delete_test.rs) | Rust | 134 | 0 | 26 | 160 |
| [backend/api/tests/routes/modules/announcements/mod.rs](/backend/api/tests/routes/modules/announcements/mod.rs) | Rust | 3 | 0 | 0 | 3 |
| [backend/api/tests/routes/modules/announcements/post\_test.rs](/backend/api/tests/routes/modules/announcements/post_test.rs) | Rust | 189 | 0 | 41 | 230 |
| [backend/api/tests/routes/modules/announcements/put\_test.rs](/backend/api/tests/routes/modules/announcements/put_test.rs) | Rust | 214 | 0 | 42 | 256 |
| [backend/api/tests/routes/modules/assignments/config/get\_test.rs](/backend/api/tests/routes/modules/assignments/config/get_test.rs) | Rust | 261 | 3 | 46 | 310 |
| [backend/api/tests/routes/modules/assignments/config/mod.rs](/backend/api/tests/routes/modules/assignments/config/mod.rs) | Rust | 2 | 0 | 0 | 2 |
| [backend/api/tests/routes/modules/assignments/config/post\_test.rs](/backend/api/tests/routes/modules/assignments/config/post_test.rs) | Rust | 258 | 4 | 41 | 303 |
| [backend/api/tests/routes/modules/assignments/delete\_test.rs](/backend/api/tests/routes/modules/assignments/delete_test.rs) | Rust | 398 | 5 | 70 | 473 |
| [backend/api/tests/routes/modules/assignments/files/delete\_test.rs](/backend/api/tests/routes/modules/assignments/files/delete_test.rs) | Rust | 178 | 0 | 24 | 202 |
| [backend/api/tests/routes/modules/assignments/files/get\_test.rs](/backend/api/tests/routes/modules/assignments/files/get_test.rs) | Rust | 222 | 0 | 38 | 260 |
| [backend/api/tests/routes/modules/assignments/files/mod.rs](/backend/api/tests/routes/modules/assignments/files/mod.rs) | Rust | 3 | 0 | 0 | 3 |
| [backend/api/tests/routes/modules/assignments/files/post\_test.rs](/backend/api/tests/routes/modules/assignments/files/post_test.rs) | Rust | 207 | 0 | 31 | 238 |
| [backend/api/tests/routes/modules/assignments/get\_test.rs](/backend/api/tests/routes/modules/assignments/get_test.rs) | Rust | 472 | 4 | 94 | 570 |
| [backend/api/tests/routes/modules/assignments/mark\_allocator/get\_test.rs](/backend/api/tests/routes/modules/assignments/mark_allocator/get_test.rs) | Rust | 136 | 0 | 23 | 159 |
| [backend/api/tests/routes/modules/assignments/mark\_allocator/mod.rs](/backend/api/tests/routes/modules/assignments/mark_allocator/mod.rs) | Rust | 3 | 0 | 0 | 3 |
| [backend/api/tests/routes/modules/assignments/mark\_allocator/post\_test.rs](/backend/api/tests/routes/modules/assignments/mark_allocator/post_test.rs) | Rust | 223 | 0 | 29 | 252 |
| [backend/api/tests/routes/modules/assignments/mark\_allocator/put\_test.rs](/backend/api/tests/routes/modules/assignments/mark_allocator/put_test.rs) | Rust | 219 | 0 | 37 | 256 |
| [backend/api/tests/routes/modules/assignments/memo\_output/get\_test.rs](/backend/api/tests/routes/modules/assignments/memo_output/get_test.rs) | Rust | 239 | 1 | 36 | 276 |
| [backend/api/tests/routes/modules/assignments/memo\_output/mod.rs](/backend/api/tests/routes/modules/assignments/memo_output/mod.rs) | Rust | 2 | 0 | 0 | 2 |
| [backend/api/tests/routes/modules/assignments/memo\_output/post\_test.rs](/backend/api/tests/routes/modules/assignments/memo_output/post_test.rs) | Rust | 333 | 0 | 39 | 372 |
| [backend/api/tests/routes/modules/assignments/mod.rs](/backend/api/tests/routes/modules/assignments/mod.rs) | Rust | 12 | 0 | 0 | 12 |
| [backend/api/tests/routes/modules/assignments/plagiarism/delete\_test.rs](/backend/api/tests/routes/modules/assignments/plagiarism/delete_test.rs) | Rust | 430 | 18 | 54 | 502 |
| [backend/api/tests/routes/modules/assignments/plagiarism/get\_test.rs](/backend/api/tests/routes/modules/assignments/plagiarism/get_test.rs) | Rust | 483 | 16 | 90 | 589 |
| [backend/api/tests/routes/modules/assignments/plagiarism/mod.rs](/backend/api/tests/routes/modules/assignments/plagiarism/mod.rs) | Rust | 5 | 0 | 0 | 5 |
| [backend/api/tests/routes/modules/assignments/plagiarism/patch\_test.rs](/backend/api/tests/routes/modules/assignments/plagiarism/patch_test.rs) | Rust | 384 | 14 | 57 | 455 |
| [backend/api/tests/routes/modules/assignments/plagiarism/post\_test.rs](/backend/api/tests/routes/modules/assignments/plagiarism/post_test.rs) | Rust | 339 | 10 | 54 | 403 |
| [backend/api/tests/routes/modules/assignments/plagiarism/put\_test.rs](/backend/api/tests/routes/modules/assignments/plagiarism/put_test.rs) | Rust | 320 | 6 | 45 | 371 |
| [backend/api/tests/routes/modules/assignments/post\_test.rs](/backend/api/tests/routes/modules/assignments/post_test.rs) | Rust | 211 | 0 | 28 | 239 |
| [backend/api/tests/routes/modules/assignments/put\_test.rs](/backend/api/tests/routes/modules/assignments/put_test.rs) | Rust | 1,117 | 30 | 212 | 1,359 |
| [backend/api/tests/routes/modules/assignments/submissions/get\_test.rs](/backend/api/tests/routes/modules/assignments/submissions/get_test.rs) | Rust | 301 | 2 | 50 | 353 |
| [backend/api/tests/routes/modules/assignments/submissions/mod.rs](/backend/api/tests/routes/modules/assignments/submissions/mod.rs) | Rust | 2 | 0 | 0 | 2 |
| [backend/api/tests/routes/modules/assignments/submissions/post\_test.rs](/backend/api/tests/routes/modules/assignments/submissions/post_test.rs) | Rust | 1,800 | 96 | 240 | 2,136 |
| [backend/api/tests/routes/modules/assignments/tasks/delete\_test.rs](/backend/api/tests/routes/modules/assignments/tasks/delete_test.rs) | Rust | 158 | 4 | 22 | 184 |
| [backend/api/tests/routes/modules/assignments/tasks/get\_test.rs](/backend/api/tests/routes/modules/assignments/tasks/get_test.rs) | Rust | 349 | 12 | 57 | 418 |
| [backend/api/tests/routes/modules/assignments/tasks/mod.rs](/backend/api/tests/routes/modules/assignments/tasks/mod.rs) | Rust | 4 | 0 | 0 | 4 |
| [backend/api/tests/routes/modules/assignments/tasks/post\_test.rs](/backend/api/tests/routes/modules/assignments/tasks/post_test.rs) | Rust | 301 | 10 | 47 | 358 |
| [backend/api/tests/routes/modules/assignments/tasks/put\_test.rs](/backend/api/tests/routes/modules/assignments/tasks/put_test.rs) | Rust | 312 | 10 | 47 | 369 |
| [backend/api/tests/routes/modules/assignments/tickets/delete\_test.rs](/backend/api/tests/routes/modules/assignments/tickets/delete_test.rs) | Rust | 113 | 0 | 11 | 124 |
| [backend/api/tests/routes/modules/assignments/tickets/mod.rs](/backend/api/tests/routes/modules/assignments/tickets/mod.rs) | Rust | 4 | 0 | 0 | 4 |
| [backend/api/tests/routes/modules/assignments/tickets/post\_test.rs](/backend/api/tests/routes/modules/assignments/tickets/post_test.rs) | Rust | 104 | 0 | 19 | 123 |
| [backend/api/tests/routes/modules/assignments/tickets/put\_test.rs](/backend/api/tests/routes/modules/assignments/tickets/put_test.rs) | Rust | 151 | 0 | 22 | 173 |
| [backend/api/tests/routes/modules/assignments/tickets/ticket\_messages/delete\_test.rs](/backend/api/tests/routes/modules/assignments/tickets/ticket_messages/delete_test.rs) | Rust | 237 | 0 | 33 | 270 |
| [backend/api/tests/routes/modules/assignments/tickets/ticket\_messages/mod.rs](/backend/api/tests/routes/modules/assignments/tickets/ticket_messages/mod.rs) | Rust | 3 | 0 | 0 | 3 |
| [backend/api/tests/routes/modules/assignments/tickets/ticket\_messages/post\_test.rs](/backend/api/tests/routes/modules/assignments/tickets/ticket_messages/post_test.rs) | Rust | 222 | 0 | 31 | 253 |
| [backend/api/tests/routes/modules/assignments/tickets/ticket\_messages/put\_test.rs](/backend/api/tests/routes/modules/assignments/tickets/ticket_messages/put_test.rs) | Rust | 250 | 0 | 34 | 284 |
| [backend/api/tests/routes/modules/delete\_test.rs](/backend/api/tests/routes/modules/delete_test.rs) | Rust | 277 | 10 | 53 | 340 |
| [backend/api/tests/routes/modules/get\_test.rs](/backend/api/tests/routes/modules/get_test.rs) | Rust | 297 | 11 | 63 | 371 |
| [backend/api/tests/routes/modules/mod.rs](/backend/api/tests/routes/modules/mod.rs) | Rust | 7 | 0 | 0 | 7 |
| [backend/api/tests/routes/modules/personnel/delete\_test.rs](/backend/api/tests/routes/modules/personnel/delete_test.rs) | Rust | 165 | 0 | 31 | 196 |
| [backend/api/tests/routes/modules/personnel/get\_test.rs](/backend/api/tests/routes/modules/personnel/get_test.rs) | Rust | 145 | 0 | 35 | 180 |
| [backend/api/tests/routes/modules/personnel/mod.rs](/backend/api/tests/routes/modules/personnel/mod.rs) | Rust | 3 | 0 | 0 | 3 |
| [backend/api/tests/routes/modules/personnel/post\_test.rs](/backend/api/tests/routes/modules/personnel/post_test.rs) | Rust | 150 | 1 | 33 | 184 |
| [backend/api/tests/routes/modules/post\_test.rs](/backend/api/tests/routes/modules/post_test.rs) | Rust | 223 | 9 | 42 | 274 |
| [backend/api/tests/routes/modules/put\_test.rs](/backend/api/tests/routes/modules/put_test.rs) | Rust | 474 | 18 | 81 | 573 |
| [backend/api/tests/routes/users/delete\_test.rs](/backend/api/tests/routes/users/delete_test.rs) | Rust | 158 | 7 | 32 | 197 |
| [backend/api/tests/routes/users/get\_test.rs](/backend/api/tests/routes/users/get_test.rs) | Rust | 262 | 15 | 56 | 333 |
| [backend/api/tests/routes/users/mod.rs](/backend/api/tests/routes/users/mod.rs) | Rust | 4 | 0 | 0 | 4 |
| [backend/api/tests/routes/users/post\_test.rs](/backend/api/tests/routes/users/post_test.rs) | Rust | 163 | 6 | 34 | 203 |
| [backend/api/tests/routes/users/put\_test.rs](/backend/api/tests/routes/users/put_test.rs) | Rust | 207 | 10 | 40 | 257 |
| [backend/api/tests/ws/chat\_handler\_test.rs](/backend/api/tests/ws/chat_handler_test.rs) | Rust | 95 | 0 | 19 | 114 |
| [backend/api/tests/ws/mod.rs](/backend/api/tests/ws/mod.rs) | Rust | 1 | 0 | 0 | 1 |
| [backend/code\_manager/Cargo.toml](/backend/code_manager/Cargo.toml) | TOML | 20 | 0 | 2 | 22 |
| [backend/code\_manager/images/Dockerfile](/backend/code_manager/images/Dockerfile) | Docker | 13 | 0 | 3 | 16 |
| [backend/code\_manager/src/api/api.rs](/backend/code_manager/src/api/api.rs) | Rust | 49 | 3 | 11 | 63 |
| [backend/code\_manager/src/api/mod.rs](/backend/code_manager/src/api/mod.rs) | Rust | 1 | 1 | 1 | 3 |
| [backend/code\_manager/src/container/container.rs](/backend/code_manager/src/container/container.rs) | Rust | 153 | 1 | 35 | 189 |
| [backend/code\_manager/src/container/mod.rs](/backend/code_manager/src/container/mod.rs) | Rust | 1 | 1 | 1 | 3 |
| [backend/code\_manager/src/lib.rs](/backend/code_manager/src/lib.rs) | Rust | 4 | 1 | 1 | 6 |
| [backend/code\_manager/src/main.rs](/backend/code_manager/src/main.rs) | Rust | 30 | 7 | 7 | 44 |
| [backend/code\_manager/src/manager/manager.rs](/backend/code_manager/src/manager/manager.rs) | Rust | 73 | 5 | 16 | 94 |
| [backend/code\_manager/src/manager/mod.rs](/backend/code_manager/src/manager/mod.rs) | Rust | 2 | 1 | 1 | 4 |
| [backend/code\_manager/src/manager/queue.rs](/backend/code_manager/src/manager/queue.rs) | Rust | 34 | 4 | 6 | 44 |
| [backend/code\_manager/src/utils/compression.rs](/backend/code_manager/src/utils/compression.rs) | Rust | 123 | 11 | 20 | 154 |
| [backend/code\_manager/src/utils/config\_management.rs](/backend/code_manager/src/utils/config_management.rs) | Rust | 16 | 10 | 5 | 31 |
| [backend/code\_manager/src/utils/mod.rs](/backend/code_manager/src/utils/mod.rs) | Rust | 2 | 1 | 1 | 4 |
| [backend/code\_manager/tests/queue.rs](/backend/code_manager/tests/queue.rs) | Rust | 205 | 1 | 36 | 242 |
| [backend/code\_manager/tests/run\_code.rs](/backend/code_manager/tests/run_code.rs) | Rust | 44 | 1 | 12 | 57 |
| [backend/code\_runner/Cargo.toml](/backend/code_runner/Cargo.toml) | TOML | 28 | 0 | 5 | 33 |
| [backend/code\_runner/src/lib.rs](/backend/code_runner/src/lib.rs) | Rust | 597 | 105 | 128 | 830 |
| [backend/code\_runner/src/mod.rs](/backend/code_runner/src/mod.rs) | Rust | 2 | 0 | 1 | 3 |
| [backend/code\_runner/src/validate\_files.rs](/backend/code_runner/src/validate_files.rs) | Rust | 295 | 88 | 58 | 441 |
| [backend/code\_runner/tests/interpreter.rs](/backend/code_runner/tests/interpreter.rs) | Rust | 209 | 4 | 29 | 242 |
| [backend/code\_runner/tests/memo\_outputs.rs](/backend/code_runner/tests/memo_outputs.rs) | Rust | 98 | 0 | 21 | 119 |
| [backend/code\_runner/tests/submission\_outputs.rs](/backend/code_runner/tests/submission_outputs.rs) | Rust | 183 | 0 | 34 | 217 |
| [backend/common/Cargo.toml](/backend/common/Cargo.toml) | TOML | 8 | 0 | 1 | 9 |
| [backend/common/src/lib.rs](/backend/common/src/lib.rs) | Rust | 12 | 0 | 1 | 13 |
| [backend/db/Cargo.toml](/backend/db/Cargo.toml) | TOML | 20 | 0 | 2 | 22 |
| [backend/db/src/lib.rs](/backend/db/src/lib.rs) | Rust | 12 | 0 | 3 | 15 |
| [backend/db/src/models/announcements.rs](/backend/db/src/models/announcements.rs) | Rust | 93 | 0 | 17 | 110 |
| [backend/db/src/models/assignment.rs](/backend/db/src/models/assignment.rs) | Rust | 462 | 54 | 59 | 575 |
| [backend/db/src/models/assignment\_file.rs](/backend/db/src/models/assignment_file.rs) | Rust | 225 | 21 | 44 | 290 |
| [backend/db/src/models/assignment\_interpreter.rs](/backend/db/src/models/assignment_interpreter.rs) | Rust | 102 | 76 | 40 | 218 |
| [backend/db/src/models/assignment\_memo\_output.rs](/backend/db/src/models/assignment_memo_output.rs) | Rust | 109 | 26 | 26 | 161 |
| [backend/db/src/models/assignment\_overwrite\_file.rs](/backend/db/src/models/assignment_overwrite_file.rs) | Rust | 112 | 4 | 24 | 140 |
| [backend/db/src/models/assignment\_submission.rs](/backend/db/src/models/assignment_submission.rs) | Rust | 249 | 83 | 41 | 373 |
| [backend/db/src/models/assignment\_submission\_output.rs](/backend/db/src/models/assignment_submission_output.rs) | Rust | 193 | 11 | 34 | 238 |
| [backend/db/src/models/assignment\_task.rs](/backend/db/src/models/assignment_task.rs) | Rust | 83 | 6 | 8 | 97 |
| [backend/db/src/models/mod.rs](/backend/db/src/models/mod.rs) | Rust | 32 | 0 | 2 | 34 |
| [backend/db/src/models/module.rs](/backend/db/src/models/module.rs) | Rust | 128 | 18 | 27 | 173 |
| [backend/db/src/models/password\_reset\_token.rs](/backend/db/src/models/password_reset_token.rs) | Rust | 79 | 0 | 9 | 88 |
| [backend/db/src/models/plagiarism\_case.rs](/backend/db/src/models/plagiarism_case.rs) | Rust | 64 | 29 | 16 | 109 |
| [backend/db/src/models/ticket\_messages.rs](/backend/db/src/models/ticket_messages.rs) | Rust | 87 | 0 | 18 | 105 |
| [backend/db/src/models/tickets.rs](/backend/db/src/models/tickets.rs) | Rust | 115 | 0 | 27 | 142 |
| [backend/db/src/models/user.rs](/backend/db/src/models/user.rs) | Rust | 241 | 68 | 39 | 348 |
| [backend/db/src/models/user\_module\_role.rs](/backend/db/src/models/user_module_role.rs) | Rust | 164 | 19 | 28 | 211 |
| [backend/db/src/test\_utils.rs](/backend/db/src/test_utils.rs) | Rust | 12 | 0 | 4 | 16 |
| [backend/marker/Cargo.toml](/backend/marker/Cargo.toml) | TOML | 14 | 0 | 2 | 16 |
| [backend/marker/src/comparators/exact\_comparator.rs](/backend/marker/src/comparators/exact_comparator.rs) | Rust | 126 | 24 | 14 | 164 |
| [backend/marker/src/comparators/mod.rs](/backend/marker/src/comparators/mod.rs) | Rust | 3 | 14 | 1 | 18 |
| [backend/marker/src/comparators/percentage\_comparator.rs](/backend/marker/src/comparators/percentage_comparator.rs) | Rust | 151 | 32 | 22 | 205 |
| [backend/marker/src/comparators/regex\_comparator.rs](/backend/marker/src/comparators/regex_comparator.rs) | Rust | 163 | 25 | 22 | 210 |
| [backend/marker/src/error.rs](/backend/marker/src/error.rs) | Rust | 12 | 31 | 1 | 44 |
| [backend/marker/src/feedback/ai\_feedback.rs](/backend/marker/src/feedback/ai_feedback.rs) | Rust | 145 | 57 | 27 | 229 |
| [backend/marker/src/feedback/auto\_feedback.rs](/backend/marker/src/feedback/auto_feedback.rs) | Rust | 97 | 18 | 12 | 127 |
| [backend/marker/src/feedback/manual\_feedback.rs](/backend/marker/src/feedback/manual_feedback.rs) | Rust | 11 | 4 | 3 | 18 |
| [backend/marker/src/feedback/mod.rs](/backend/marker/src/feedback/mod.rs) | Rust | 3 | 11 | 1 | 15 |
| [backend/marker/src/lib.rs](/backend/marker/src/lib.rs) | Rust | 585 | 76 | 86 | 747 |
| [backend/marker/src/parsers/allocator\_parser.rs](/backend/marker/src/parsers/allocator_parser.rs) | Rust | 258 | 55 | 27 | 340 |
| [backend/marker/src/parsers/complexity\_parser.rs](/backend/marker/src/parsers/complexity_parser.rs) | Rust | 187 | 71 | 29 | 287 |
| [backend/marker/src/parsers/coverage\_parser.rs](/backend/marker/src/parsers/coverage_parser.rs) | Rust | 270 | 86 | 40 | 396 |
| [backend/marker/src/parsers/mod.rs](/backend/marker/src/parsers/mod.rs) | Rust | 4 | 14 | 1 | 19 |
| [backend/marker/src/parsers/output\_parser.rs](/backend/marker/src/parsers/output_parser.rs) | Rust | 378 | 47 | 46 | 471 |
| [backend/marker/src/report.rs](/backend/marker/src/report.rs) | Rust | 191 | 122 | 22 | 335 |
| [backend/marker/src/scorer.rs](/backend/marker/src/scorer.rs) | Rust | 119 | 50 | 9 | 178 |
| [backend/marker/src/test\_files/allocator\_parser/allocator\_report\_1.json](/backend/marker/src/test_files/allocator_parser/allocator_report_1.json) | JSON | 15 | 0 | 1 | 16 |
| [backend/marker/src/test\_files/allocator\_parser/allocator\_report\_2.json](/backend/marker/src/test_files/allocator_parser/allocator_report_2.json) | JSON | 22 | 0 | 1 | 23 |
| [backend/marker/src/test\_files/allocator\_parser/allocator\_report\_3.json](/backend/marker/src/test_files/allocator_parser/allocator_report_3.json) | JSON | 13 | 0 | 1 | 14 |
| [backend/marker/src/test\_files/allocator\_parser/allocator\_report\_4.json](/backend/marker/src/test_files/allocator_parser/allocator_report_4.json) | JSON | 15 | 0 | 1 | 16 |
| [backend/marker/src/test\_files/allocator\_parser/allocator\_report\_5.json](/backend/marker/src/test_files/allocator_parser/allocator_report_5.json) | JSON | 15 | 0 | 1 | 16 |
| [backend/marker/src/test\_files/complexity\_parser/complexity\_report\_1.json](/backend/marker/src/test_files/complexity_parser/complexity_report_1.json) | JSON | 9 | 0 | 0 | 9 |
| [backend/marker/src/test\_files/complexity\_parser/complexity\_report\_2.json](/backend/marker/src/test_files/complexity_parser/complexity_report_2.json) | JSON | 9 | 0 | 0 | 9 |
| [backend/marker/src/test\_files/complexity\_parser/complexity\_report\_3.json](/backend/marker/src/test_files/complexity_parser/complexity_report_3.json) | JSON | 8 | 0 | 0 | 8 |
| [backend/marker/src/test\_files/complexity\_parser/complexity\_report\_4.json](/backend/marker/src/test_files/complexity_parser/complexity_report_4.json) | JSON | 9 | 0 | 0 | 9 |
| [backend/marker/src/test\_files/complexity\_parser/complexity\_report\_5.json](/backend/marker/src/test_files/complexity_parser/complexity_report_5.json) | JSON | 12 | 0 | 0 | 12 |
| [backend/marker/src/test\_files/coverage\_parser/coverage\_report\_1.json](/backend/marker/src/test_files/coverage_parser/coverage_report_1.json) | JSON | 17 | 0 | 0 | 17 |
| [backend/marker/src/test\_files/coverage\_parser/coverage\_report\_2.json](/backend/marker/src/test_files/coverage_parser/coverage_report_2.json) | JSON | 28 | 0 | 0 | 28 |
| [backend/marker/src/test\_files/coverage\_parser/coverage\_report\_3.json](/backend/marker/src/test_files/coverage_parser/coverage_report_3.json) | JSON | 11 | 0 | 0 | 11 |
| [backend/marker/src/test\_files/coverage\_parser/coverage\_report\_4.json](/backend/marker/src/test_files/coverage_parser/coverage_report_4.json) | JSON | 17 | 0 | 0 | 17 |
| [backend/marker/src/test\_files/coverage\_parser/coverage\_report\_5.json](/backend/marker/src/test_files/coverage_parser/coverage_report_5.json) | JSON | 16 | 0 | 0 | 16 |
| [backend/marker/src/test\_files/file\_loader/case1/allocator.json](/backend/marker/src/test_files/file_loader/case1/allocator.json) | JSON | 1 | 0 | 0 | 1 |
| [backend/marker/src/test\_files/file\_loader/case1/complexity.json](/backend/marker/src/test_files/file_loader/case1/complexity.json) | JSON | 1 | 0 | 0 | 1 |
| [backend/marker/src/test\_files/file\_loader/case1/coverage.json](/backend/marker/src/test_files/file_loader/case1/coverage.json) | JSON | 1 | 0 | 0 | 1 |
| [backend/marker/src/test\_files/file\_loader/case2/allocator.json](/backend/marker/src/test_files/file_loader/case2/allocator.json) | JSON | 1 | 0 | 0 | 1 |
| [backend/marker/src/test\_files/file\_loader/case2/complexity.json](/backend/marker/src/test_files/file_loader/case2/complexity.json) | JSON | 1 | 0 | 0 | 1 |
| [backend/marker/src/test\_files/file\_loader/case2/coverage.json](/backend/marker/src/test_files/file_loader/case2/coverage.json) | JSON | 1 | 0 | 0 | 1 |
| [backend/marker/src/test\_files/file\_loader/case3/allocator.json](/backend/marker/src/test_files/file_loader/case3/allocator.json) | JSON | 1 | 0 | 0 | 1 |
| [backend/marker/src/test\_files/file\_loader/case3/complexity.json](/backend/marker/src/test_files/file_loader/case3/complexity.json) | JSON | 1 | 0 | 0 | 1 |
| [backend/marker/src/test\_files/file\_loader/case3/coverage.json](/backend/marker/src/test_files/file_loader/case3/coverage.json) | JSON | 1 | 0 | 0 | 1 |
| [backend/marker/src/test\_files/file\_loader/case4/allocator.json](/backend/marker/src/test_files/file_loader/case4/allocator.json) | JSON | 1 | 0 | 0 | 1 |
| [backend/marker/src/test\_files/file\_loader/case4/complexity.json](/backend/marker/src/test_files/file_loader/case4/complexity.json) | JSON | 1 | 0 | 0 | 1 |
| [backend/marker/src/test\_files/file\_loader/case4/coverage.json](/backend/marker/src/test_files/file_loader/case4/coverage.json) | JSON | 1 | 0 | 0 | 1 |
| [backend/marker/src/test\_files/file\_loader/case5/allocator.json](/backend/marker/src/test_files/file_loader/case5/allocator.json) | JSON | 1 | 0 | 0 | 1 |
| [backend/marker/src/test\_files/file\_loader/case5/complexity.json](/backend/marker/src/test_files/file_loader/case5/complexity.json) | JSON | 1 | 0 | 0 | 1 |
| [backend/marker/src/test\_files/file\_loader/case5/coverage.json](/backend/marker/src/test_files/file_loader/case5/coverage.json) | JSON | 1 | 0 | 0 | 1 |
| [backend/marker/src/test\_files/marker/case1/allocator.json](/backend/marker/src/test_files/marker/case1/allocator.json) | JSON | 14 | 0 | 0 | 14 |
| [backend/marker/src/test\_files/marker/case2/allocator.json](/backend/marker/src/test_files/marker/case2/allocator.json) | JSON | 25 | 0 | 1 | 26 |
| [backend/marker/src/test\_files/marker/case3/allocator.json](/backend/marker/src/test_files/marker/case3/allocator.json) | JSON | 25 | 0 | 0 | 25 |
| [backend/marker/src/test\_files/marker/case4/allocator.json](/backend/marker/src/test_files/marker/case4/allocator.json) | JSON | 25 | 0 | 0 | 25 |
| [backend/marker/src/test\_files/marker/case5/allocator.json](/backend/marker/src/test_files/marker/case5/allocator.json) | JSON | 25 | 0 | 0 | 25 |
| [backend/marker/src/test\_files/marker/case6/allocator.json](/backend/marker/src/test_files/marker/case6/allocator.json) | JSON | 15 | 0 | 0 | 15 |
| [backend/marker/src/test\_files/marker/case7/allocator.json](/backend/marker/src/test_files/marker/case7/allocator.json) | JSON | 15 | 0 | 0 | 15 |
| [backend/marker/src/test\_files/marker/case8/allocator.json](/backend/marker/src/test_files/marker/case8/allocator.json) | JSON | 15 | 0 | 0 | 15 |
| [backend/marker/src/test\_files/marker/case9/allocator.json](/backend/marker/src/test_files/marker/case9/allocator.json) | JSON | 15 | 0 | 0 | 15 |
| [backend/marker/src/traits/comparator.rs](/backend/marker/src/traits/comparator.rs) | Rust | 9 | 10 | 1 | 20 |
| [backend/marker/src/traits/feedback.rs](/backend/marker/src/traits/feedback.rs) | Rust | 13 | 19 | 3 | 35 |
| [backend/marker/src/traits/mod.rs](/backend/marker/src/traits/mod.rs) | Rust | 3 | 10 | 1 | 14 |
| [backend/marker/src/traits/parser.rs](/backend/marker/src/traits/parser.rs) | Rust | 5 | 49 | 4 | 58 |
| [backend/marker/src/types.rs](/backend/marker/src/types.rs) | Rust | 30 | 29 | 6 | 65 |
| [backend/marker/src/utilities/file\_loader.rs](/backend/marker/src/utilities/file_loader.rs) | Rust | 237 | 54 | 20 | 311 |
| [backend/marker/src/utilities/mod.rs](/backend/marker/src/utilities/mod.rs) | Rust | 1 | 8 | 1 | 10 |
| [backend/migration/Cargo.toml](/backend/migration/Cargo.toml) | TOML | 18 | 0 | 3 | 21 |
| [backend/migration/src/lib.rs](/backend/migration/src/lib.rs) | Rust | 4 | 0 | 2 | 6 |
| [backend/migration/src/main.rs](/backend/migration/src/main.rs) | Rust | 45 | 0 | 8 | 53 |
| [backend/migration/src/migrations/m202505290001\_create\_users.rs](/backend/migration/src/migrations/m202505290001_create_users.rs) | Rust | 33 | 0 | 5 | 38 |
| [backend/migration/src/migrations/m202505290002\_create\_modules.rs](/backend/migration/src/migrations/m202505290002_create_modules.rs) | Rust | 33 | 0 | 5 | 38 |
| [backend/migration/src/migrations/m202505290003\_create\_user\_module\_roles.rs](/backend/migration/src/migrations/m202505290003_create_user_module_roles.rs) | Rust | 55 | 0 | 5 | 60 |
| [backend/migration/src/migrations/m202505290004\_create\_assignments.rs](/backend/migration/src/migrations/m202505290004_create_assignments.rs) | Rust | 76 | 0 | 5 | 81 |
| [backend/migration/src/migrations/m202505290005\_create\_assignment\_files.rs](/backend/migration/src/migrations/m202505290005_create_assignment_files.rs) | Rust | 76 | 0 | 5 | 81 |
| [backend/migration/src/migrations/m202505290006\_create\_assignment\_submissions.rs](/backend/migration/src/migrations/m202505290006_create_assignment_submissions.rs) | Rust | 76 | 0 | 5 | 81 |
| [backend/migration/src/migrations/m202505290007\_create\_password\_reset\_tokens.rs](/backend/migration/src/migrations/m202505290007_create_password_reset_tokens.rs) | Rust | 62 | 0 | 5 | 67 |
| [backend/migration/src/migrations/m202505290008\_create\_tasks.rs](/backend/migration/src/migrations/m202505290008_create_tasks.rs) | Rust | 71 | 0 | 5 | 76 |
| [backend/migration/src/migrations/m202505290009\_create\_memo\_outputs.rs](/backend/migration/src/migrations/m202505290009_create_memo_outputs.rs) | Rust | 74 | 0 | 5 | 79 |
| [backend/migration/src/migrations/m202505290010\_create\_submission\_outputs.rs](/backend/migration/src/migrations/m202505290010_create_submission_outputs.rs) | Rust | 77 | 0 | 5 | 82 |
| [backend/migration/src/migrations/m202505290011\_create\_overwrite\_files.rs](/backend/migration/src/migrations/m202505290011_create_overwrite_files.rs) | Rust | 78 | 0 | 5 | 83 |
| [backend/migration/src/migrations/m202506080012\_create\_plagiarism\_cases.rs](/backend/migration/src/migrations/m202506080012_create_plagiarism_cases.rs) | Rust | 72 | 0 | 5 | 77 |
| [backend/migration/src/migrations/m202508020001\_create\_tickets.rs](/backend/migration/src/migrations/m202508020001_create_tickets.rs) | Rust | 92 | 0 | 5 | 97 |
| [backend/migration/src/migrations/m202508020002\_create\_ticket\_messages.rs](/backend/migration/src/migrations/m202508020002_create_ticket_messages.rs) | Rust | 75 | 0 | 5 | 80 |
| [backend/migration/src/migrations/m202508020003\_create\_interpreter.rs](/backend/migration/src/migrations/m202508020003_create_interpreter.rs) | Rust | 58 | 0 | 5 | 63 |
| [backend/migration/src/migrations/m202508060001\_create\_announcements.rs](/backend/migration/src/migrations/m202508060001_create_announcements.rs) | Rust | 86 | 0 | 5 | 91 |
| [backend/migration/src/migrations/mod.rs](/backend/migration/src/migrations/mod.rs) | Rust | 16 | 0 | 1 | 17 |
| [backend/migration/src/migrator.rs](/backend/migration/src/migrator.rs) | Rust | 26 | 0 | 4 | 30 |
| [backend/migration/src/runner.rs](/backend/migration/src/runner.rs) | Rust | 39 | 0 | 8 | 47 |
| [backend/moss\_parser/Cargo.toml](/backend/moss_parser/Cargo.toml) | TOML | 14 | 0 | 2 | 16 |
| [backend/moss\_parser/src/main.rs](/backend/moss_parser/src/main.rs) | Rust | 336 | 5 | 48 | 389 |
| [backend/seeder/Cargo.toml](/backend/seeder/Cargo.toml) | TOML | 18 | 0 | 1 | 19 |
| [backend/seeder/src/main.rs](/backend/seeder/src/main.rs) | Rust | 49 | 0 | 4 | 53 |
| [backend/seeder/src/seed.rs](/backend/seeder/src/seed.rs) | Rust | 27 | 0 | 6 | 33 |
| [backend/seeder/src/seeds/assignment.rs](/backend/seeder/src/seeds/assignment.rs) | Rust | 66 | 0 | 8 | 74 |
| [backend/seeder/src/seeds/assignment\_file.rs](/backend/seeder/src/seeds/assignment_file.rs) | Rust | 434 | 3 | 69 | 506 |
| [backend/seeder/src/seeds/assignment\_interpreter.rs](/backend/seeder/src/seeds/assignment_interpreter.rs) | Rust | 147 | 7 | 31 | 185 |
| [backend/seeder/src/seeds/assignment\_memo\_output.rs](/backend/seeder/src/seeds/assignment_memo_output.rs) | Rust | 56 | 3 | 8 | 67 |
| [backend/seeder/src/seeds/assignment\_overwrite\_file.rs](/backend/seeder/src/seeds/assignment_overwrite_file.rs) | Rust | 56 | 3 | 8 | 67 |
| [backend/seeder/src/seeds/assignment\_submission.rs](/backend/seeder/src/seeds/assignment_submission.rs) | Rust | 245 | 1 | 33 | 279 |
| [backend/seeder/src/seeds/assignment\_submission\_output.rs](/backend/seeder/src/seeds/assignment_submission_output.rs) | Rust | 57 | 1 | 9 | 67 |
| [backend/seeder/src/seeds/assignment\_task.rs](/backend/seeder/src/seeds/assignment_task.rs) | Rust | 85 | 2 | 13 | 100 |
| [backend/seeder/src/seeds/mod.rs](/backend/seeder/src/seeds/mod.rs) | Rust | 13 | 0 | 0 | 13 |
| [backend/seeder/src/seeds/module.rs](/backend/seeder/src/seeds/module.rs) | Rust | 78 | 1 | 11 | 90 |
| [backend/seeder/src/seeds/plagiarism\_case.rs](/backend/seeder/src/seeds/plagiarism_case.rs) | Rust | 65 | 4 | 12 | 81 |
| [backend/seeder/src/seeds/tickets.rs](/backend/seeder/src/seeds/tickets.rs) | Rust | 51 | 0 | 11 | 62 |
| [backend/seeder/src/seeds/user.rs](/backend/seeder/src/seeds/user.rs) | Rust | 20 | 6 | 8 | 34 |
| [backend/seeder/src/seeds/user\_role.rs](/backend/seeder/src/seeds/user_role.rs) | Rust | 60 | 3 | 8 | 71 |
| [backend/util/Cargo.toml](/backend/util/Cargo.toml) | TOML | 21 | 0 | 1 | 22 |
| [backend/util/src/config.rs](/backend/util/src/config.rs) | Rust | 150 | 23 | 28 | 201 |
| [backend/util/src/execution\_config/execution\_config.rs](/backend/util/src/execution_config/execution_config.rs) | Rust | 369 | 7 | 78 | 454 |
| [backend/util/src/execution\_config/mod.rs](/backend/util/src/execution_config/mod.rs) | Rust | 2 | 0 | 2 | 4 |
| [backend/util/src/lib.rs](/backend/util/src/lib.rs) | Rust | 5 | 0 | 0 | 5 |
| [backend/util/src/mark\_allocator/mark\_allocator.rs](/backend/util/src/mark_allocator/mark_allocator.rs) | Rust | 138 | 50 | 34 | 222 |
| [backend/util/src/mark\_allocator/mod.rs](/backend/util/src/mark_allocator/mod.rs) | Rust | 1 | 0 | 1 | 2 |
| [backend/util/src/state.rs](/backend/util/src/state.rs) | Rust | 26 | 25 | 8 | 59 |
| [backend/util/src/ws/handler.rs](/backend/util/src/ws/handler.rs) | Rust | 69 | 4 | 9 | 82 |
| [backend/util/src/ws/manager.rs](/backend/util/src/ws/manager.rs) | Rust | 80 | 39 | 26 | 145 |
| [backend/util/src/ws/mod.rs](/backend/util/src/ws/mod.rs) | Rust | 4 | 0 | 2 | 6 |
| [frontend/eslint.config.js](/frontend/eslint.config.js) | JavaScript | 51 | 1 | 3 | 55 |
| [frontend/index.html](/frontend/index.html) | HTML | 13 | 0 | 1 | 14 |
| [frontend/package-lock.json](/frontend/package-lock.json) | JSON | 12,080 | 0 | 1 | 12,081 |
| [frontend/package.json](/frontend/package.json) | JSON | 65 | 0 | 0 | 65 |
| [frontend/public/FitchFork.html](/frontend/public/FitchFork.html) | HTML | 20 | 0 | 3 | 23 |
| [frontend/public/ff\_logo\_dark.svg](/frontend/public/ff_logo_dark.svg) | XML | 4 | 0 | 1 | 5 |
| [frontend/public/ff\_logo\_favicon.svg](/frontend/public/ff_logo_favicon.svg) | XML | 5 | 0 | 1 | 6 |
| [frontend/public/ff\_logo\_light.svg](/frontend/public/ff_logo_light.svg) | XML | 4 | 0 | 1 | 5 |
| [frontend/src/App.css](/frontend/src/App.css) | PostCSS | 0 | 0 | 1 | 1 |
| [frontend/src/App.tsx](/frontend/src/App.tsx) | TypeScript JSX | 132 | 7 | 21 | 160 |
| [frontend/src/components/AdminTag.tsx](/frontend/src/components/AdminTag.tsx) | TypeScript JSX | 7 | 1 | 3 | 11 |
| [frontend/src/components/CodeDiffEditor.tsx](/frontend/src/components/CodeDiffEditor.tsx) | TypeScript JSX | 66 | 2 | 8 | 76 |
| [frontend/src/components/CodeEditor.tsx](/frontend/src/components/CodeEditor.tsx) | TypeScript JSX | 105 | 3 | 11 | 119 |
| [frontend/src/components/ControlBar.tsx](/frontend/src/components/ControlBar.tsx) | TypeScript JSX | 460 | 1 | 41 | 502 |
| [frontend/src/components/CreateModal.tsx](/frontend/src/components/CreateModal.tsx) | TypeScript JSX | 103 | 3 | 14 | 120 |
| [frontend/src/components/EditModal.tsx](/frontend/src/components/EditModal.tsx) | TypeScript JSX | 100 | 1 | 12 | 113 |
| [frontend/src/components/EntityList.tsx](/frontend/src/components/EntityList.tsx) | TypeScript JSX | 465 | 0 | 31 | 496 |
| [frontend/src/components/Logo.tsx](/frontend/src/components/Logo.tsx) | TypeScript JSX | 83 | 0 | 9 | 92 |
| [frontend/src/components/Notifier.tsx](/frontend/src/components/Notifier.tsx) | TypeScript JSX | 26 | 1 | 6 | 33 |
| [frontend/src/components/PageHeader.tsx](/frontend/src/components/PageHeader.tsx) | TypeScript JSX | 25 | 0 | 5 | 30 |
| [frontend/src/components/SettingsGroup.tsx](/frontend/src/components/SettingsGroup.tsx) | TypeScript JSX | 31 | 0 | 4 | 35 |
| [frontend/src/components/StatCard.tsx](/frontend/src/components/StatCard.tsx) | TypeScript JSX | 35 | 0 | 3 | 38 |
| [frontend/src/components/TagSummary.tsx](/frontend/src/components/TagSummary.tsx) | TypeScript JSX | 75 | 3 | 8 | 86 |
| [frontend/src/components/assignments/AssignmentCard.tsx](/frontend/src/components/assignments/AssignmentCard.tsx) | TypeScript JSX | 68 | 0 | 10 | 78 |
| [frontend/src/components/assignments/AssignmentStatusTag.tsx](/frontend/src/components/assignments/AssignmentStatusTag.tsx) | TypeScript JSX | 21 | 0 | 6 | 27 |
| [frontend/src/components/assignments/AssignmentTypeTag.tsx](/frontend/src/components/assignments/AssignmentTypeTag.tsx) | TypeScript JSX | 18 | 0 | 5 | 23 |
| [frontend/src/components/dashboard/ModuleAssignmentsPanel.tsx](/frontend/src/components/dashboard/ModuleAssignmentsPanel.tsx) | TypeScript JSX | 46 | 0 | 5 | 51 |
| [frontend/src/components/dashboard/QuickActionsPanel.tsx](/frontend/src/components/dashboard/QuickActionsPanel.tsx) | TypeScript JSX | 36 | 0 | 6 | 42 |
| [frontend/src/components/dashboard/SubmissionsPanel.tsx](/frontend/src/components/dashboard/SubmissionsPanel.tsx) | TypeScript JSX | 132 | 2 | 18 | 152 |
| [frontend/src/components/dashboard/SystemOverviewPanel.tsx](/frontend/src/components/dashboard/SystemOverviewPanel.tsx) | TypeScript JSX | 250 | 1 | 30 | 281 |
| [frontend/src/components/dashboard/UserManagementPanel.tsx](/frontend/src/components/dashboard/UserManagementPanel.tsx) | TypeScript JSX | 57 | 0 | 8 | 65 |
| [frontend/src/components/layout/HeaderBar.tsx](/frontend/src/components/layout/HeaderBar.tsx) | TypeScript JSX | 82 | 4 | 11 | 97 |
| [frontend/src/components/layout/NotificationDropdown.tsx](/frontend/src/components/layout/NotificationDropdown.tsx) | TypeScript JSX | 31 | 1 | 6 | 38 |
| [frontend/src/components/layout/SidebarContent.tsx](/frontend/src/components/layout/SidebarContent.tsx) | TypeScript JSX | 124 | 0 | 9 | 133 |
| [frontend/src/components/modules/ModuleCard.tsx](/frontend/src/components/modules/ModuleCard.tsx) | TypeScript JSX | 87 | 0 | 10 | 97 |
| [frontend/src/components/modules/ModuleCreditsTag.tsx](/frontend/src/components/modules/ModuleCreditsTag.tsx) | TypeScript JSX | 12 | 1 | 5 | 18 |
| [frontend/src/components/modules/ModuleRoleTag.tsx](/frontend/src/components/modules/ModuleRoleTag.tsx) | TypeScript JSX | 27 | 0 | 6 | 33 |
| [frontend/src/components/routes/ProtectedAdminRoute.tsx](/frontend/src/components/routes/ProtectedAdminRoute.tsx) | TypeScript JSX | 8 | 0 | 4 | 12 |
| [frontend/src/components/routes/ProtectedAuthRoute.tsx](/frontend/src/components/routes/ProtectedAuthRoute.tsx) | TypeScript JSX | 7 | 0 | 4 | 11 |
| [frontend/src/components/routes/ProtectedModuleRoute.tsx](/frontend/src/components/routes/ProtectedModuleRoute.tsx) | TypeScript JSX | 19 | 0 | 8 | 27 |
| [frontend/src/components/submissions/SubmisisonTasks.tsx](/frontend/src/components/submissions/SubmisisonTasks.tsx) | TypeScript JSX | 107 | 0 | 16 | 123 |
| [frontend/src/components/submissions/SubmissionCard.tsx](/frontend/src/components/submissions/SubmissionCard.tsx) | TypeScript JSX | 59 | 0 | 9 | 68 |
| [frontend/src/components/users/UserAdminTag.tsx](/frontend/src/components/users/UserAdminTag.tsx) | TypeScript JSX | 8 | 0 | 4 | 12 |
| [frontend/src/components/users/UserCard.tsx](/frontend/src/components/users/UserCard.tsx) | TypeScript JSX | 36 | 0 | 5 | 41 |
| [frontend/src/components/utils/ConfirmModal.tsx](/frontend/src/components/utils/ConfirmModal.tsx) | TypeScript JSX | 34 | 0 | 4 | 38 |
| [frontend/src/config/api.ts](/frontend/src/config/api.ts) | TypeScript | 2 | 3 | 0 | 5 |
| [frontend/src/constants/assignments/index.ts](/frontend/src/constants/assignments/index.ts) | TypeScript | 15 | 0 | 1 | 16 |
| [frontend/src/constants/mock/assignment.ts](/frontend/src/constants/mock/assignment.ts) | TypeScript | 5 | 0 | 1 | 6 |
| [frontend/src/constants/pagination.ts](/frontend/src/constants/pagination.ts) | TypeScript | 1 | 0 | 0 | 1 |
| [frontend/src/constants/sidebar.ts](/frontend/src/constants/sidebar.ts) | TypeScript | 89 | 9 | 8 | 106 |
| [frontend/src/context/AppMuiTheme.tsx](/frontend/src/context/AppMuiTheme.tsx) | TypeScript JSX | 16 | 1 | 4 | 21 |
| [frontend/src/context/AssignmentContext.tsx](/frontend/src/context/AssignmentContext.tsx) | TypeScript JSX | 21 | 0 | 6 | 27 |
| [frontend/src/context/AssignmentSetupContext.tsx](/frontend/src/context/AssignmentSetupContext.tsx) | TypeScript JSX | 21 | 0 | 6 | 27 |
| [frontend/src/context/AuthContext.tsx](/frontend/src/context/AuthContext.tsx) | TypeScript JSX | 203 | 2 | 32 | 237 |
| [frontend/src/context/BreadcrumbContext.tsx](/frontend/src/context/BreadcrumbContext.tsx) | TypeScript JSX | 23 | 0 | 8 | 31 |
| [frontend/src/context/ModuleContext.tsx](/frontend/src/context/ModuleContext.tsx) | TypeScript JSX | 18 | 1 | 6 | 25 |
| [frontend/src/context/StepNavigatorContext.tsx](/frontend/src/context/StepNavigatorContext.tsx) | TypeScript JSX | 13 | 0 | 4 | 17 |
| [frontend/src/context/ThemeContext.tsx](/frontend/src/context/ThemeContext.tsx) | TypeScript JSX | 43 | 1 | 10 | 54 |
| [frontend/src/context/UIContext.tsx](/frontend/src/context/UIContext.tsx) | TypeScript JSX | 43 | 1 | 9 | 53 |
| [frontend/src/hooks/useBreadcrumbs.ts](/frontend/src/hooks/useBreadcrumbs.ts) | TypeScript | 25 | 1 | 7 | 33 |
| [frontend/src/hooks/useEntityViewState.tsx](/frontend/src/hooks/useEntityViewState.tsx) | TypeScript JSX | 97 | 51 | 29 | 177 |
| [frontend/src/hooks/useNotImplemented.ts](/frontend/src/hooks/useNotImplemented.ts) | TypeScript | 8 | 0 | 4 | 12 |
| [frontend/src/hooks/useTableQuery.ts](/frontend/src/hooks/useTableQuery.ts) | TypeScript | 62 | 3 | 12 | 77 |
| [frontend/src/index.css](/frontend/src/index.css) | PostCSS | 60 | 32 | 13 | 105 |
| [frontend/src/layouts/AppLayout.tsx](/frontend/src/layouts/AppLayout.tsx) | TypeScript JSX | 126 | 0 | 18 | 144 |
| [frontend/src/layouts/AssignmentLayout.tsx](/frontend/src/layouts/AssignmentLayout.tsx) | TypeScript JSX | 470 | 2 | 41 | 513 |
| [frontend/src/layouts/AuthLayout.tsx](/frontend/src/layouts/AuthLayout.tsx) | TypeScript JSX | 8 | 0 | 2 | 10 |
| [frontend/src/layouts/HelpPageLayout.tsx](/frontend/src/layouts/HelpPageLayout.tsx) | TypeScript JSX | 49 | 0 | 7 | 56 |
| [frontend/src/layouts/ModuleLayout.tsx](/frontend/src/layouts/ModuleLayout.tsx) | TypeScript JSX | 119 | 0 | 16 | 135 |
| [frontend/src/layouts/SettingsLayout.tsx](/frontend/src/layouts/SettingsLayout.tsx) | TypeScript JSX | 60 | 0 | 7 | 67 |
| [frontend/src/main.tsx](/frontend/src/main.tsx) | TypeScript JSX | 30 | 0 | 2 | 32 |
| [frontend/src/pages/Chat.tsx](/frontend/src/pages/Chat.tsx) | TypeScript JSX | 226 | 4 | 33 | 263 |
| [frontend/src/pages/Dashboard.tsx](/frontend/src/pages/Dashboard.tsx) | TypeScript JSX | 53 | 3 | 5 | 61 |
| [frontend/src/pages/Landing.tsx](/frontend/src/pages/Landing.tsx) | TypeScript JSX | 154 | 7 | 13 | 174 |
| [frontend/src/pages/auth/Login.tsx](/frontend/src/pages/auth/Login.tsx) | TypeScript JSX | 88 | 0 | 13 | 101 |
| [frontend/src/pages/auth/PasswordResetSuccessPage.tsx](/frontend/src/pages/auth/PasswordResetSuccessPage.tsx) | TypeScript JSX | 26 | 0 | 4 | 30 |
| [frontend/src/pages/auth/RequestPasswordResetPage.tsx](/frontend/src/pages/auth/RequestPasswordResetPage.tsx) | TypeScript JSX | 91 | 0 | 15 | 106 |
| [frontend/src/pages/auth/ResetPasswordPage.tsx](/frontend/src/pages/auth/ResetPasswordPage.tsx) | TypeScript JSX | 110 | 0 | 17 | 127 |
| [frontend/src/pages/auth/Signup.tsx](/frontend/src/pages/auth/Signup.tsx) | TypeScript JSX | 121 | 0 | 15 | 136 |
| [frontend/src/pages/help/HelpAccount.tsx](/frontend/src/pages/help/HelpAccount.tsx) | TypeScript JSX | 75 | 0 | 15 | 90 |
| [frontend/src/pages/help/HelpAssignments.tsx](/frontend/src/pages/help/HelpAssignments.tsx) | TypeScript JSX | 96 | 0 | 18 | 114 |
| [frontend/src/pages/help/HelpContact.tsx](/frontend/src/pages/help/HelpContact.tsx) | TypeScript JSX | 77 | 0 | 15 | 92 |
| [frontend/src/pages/help/HelpSubmissions.tsx](/frontend/src/pages/help/HelpSubmissions.tsx) | TypeScript JSX | 104 | 0 | 17 | 121 |
| [frontend/src/pages/help/HelpTroubleshooting.tsx](/frontend/src/pages/help/HelpTroubleshooting.tsx) | TypeScript JSX | 93 | 0 | 18 | 111 |
| [frontend/src/pages/modules/ModuleGrades.tsx](/frontend/src/pages/modules/ModuleGrades.tsx) | TypeScript JSX | 250 | 5 | 31 | 286 |
| [frontend/src/pages/modules/ModuleOverview.tsx](/frontend/src/pages/modules/ModuleOverview.tsx) | TypeScript JSX | 120 | 3 | 11 | 134 |
| [frontend/src/pages/modules/ModulePersonnel.tsx](/frontend/src/pages/modules/ModulePersonnel.tsx) | TypeScript JSX | 231 | 0 | 26 | 257 |
| [frontend/src/pages/modules/ModuleResources.tsx](/frontend/src/pages/modules/ModuleResources.tsx) | TypeScript JSX | 57 | 0 | 6 | 63 |
| [frontend/src/pages/modules/ModulesList.tsx](/frontend/src/pages/modules/ModulesList.tsx) | TypeScript JSX | 290 | 0 | 19 | 309 |
| [frontend/src/pages/modules/assignments/AssignmentFiles.tsx](/frontend/src/pages/modules/assignments/AssignmentFiles.tsx) | TypeScript JSX | 124 | 0 | 14 | 138 |
| [frontend/src/pages/modules/assignments/AssignmentsList.tsx](/frontend/src/pages/modules/assignments/AssignmentsList.tsx) | TypeScript JSX | 407 | 0 | 29 | 436 |
| [frontend/src/pages/modules/assignments/Config.tsx](/frontend/src/pages/modules/assignments/Config.tsx) | TypeScript JSX | 244 | 2 | 29 | 275 |
| [frontend/src/pages/modules/assignments/MarkAllocator.tsx](/frontend/src/pages/modules/assignments/MarkAllocator.tsx) | TypeScript JSX | 60 | 0 | 12 | 72 |
| [frontend/src/pages/modules/assignments/MemoOutput.tsx](/frontend/src/pages/modules/assignments/MemoOutput.tsx) | TypeScript JSX | 51 | 0 | 6 | 57 |
| [frontend/src/pages/modules/assignments/Tasks.tsx](/frontend/src/pages/modules/assignments/Tasks.tsx) | TypeScript JSX | 287 | 1 | 30 | 318 |
| [frontend/src/pages/modules/assignments/steps/AssignmentSetup.tsx](/frontend/src/pages/modules/assignments/steps/AssignmentSetup.tsx) | TypeScript JSX | 210 | 0 | 38 | 248 |
| [frontend/src/pages/modules/assignments/steps/StepConfig.tsx](/frontend/src/pages/modules/assignments/steps/StepConfig.tsx) | TypeScript JSX | 88 | 0 | 14 | 102 |
| [frontend/src/pages/modules/assignments/steps/StepFilesResources.tsx](/frontend/src/pages/modules/assignments/steps/StepFilesResources.tsx) | TypeScript JSX | 178 | 0 | 22 | 200 |
| [frontend/src/pages/modules/assignments/steps/StepFinal.tsx](/frontend/src/pages/modules/assignments/steps/StepFinal.tsx) | TypeScript JSX | 11 | 0 | 4 | 15 |
| [frontend/src/pages/modules/assignments/steps/StepMarkAllocator.tsx](/frontend/src/pages/modules/assignments/steps/StepMarkAllocator.tsx) | TypeScript JSX | 72 | 0 | 13 | 85 |
| [frontend/src/pages/modules/assignments/steps/StepMemoAndAllocator.tsx](/frontend/src/pages/modules/assignments/steps/StepMemoAndAllocator.tsx) | TypeScript JSX | 83 | 1 | 14 | 98 |
| [frontend/src/pages/modules/assignments/steps/StepMemoOutput.tsx](/frontend/src/pages/modules/assignments/steps/StepMemoOutput.tsx) | TypeScript JSX | 72 | 0 | 12 | 84 |
| [frontend/src/pages/modules/assignments/steps/StepTasks.tsx](/frontend/src/pages/modules/assignments/steps/StepTasks.tsx) | TypeScript JSX | 172 | 0 | 24 | 196 |
| [frontend/src/pages/modules/assignments/steps/StepWelcome.tsx](/frontend/src/pages/modules/assignments/steps/StepWelcome.tsx) | TypeScript JSX | 35 | 0 | 7 | 42 |
| [frontend/src/pages/modules/assignments/submissions/index/Submissions.tsx](/frontend/src/pages/modules/assignments/submissions/index/Submissions.tsx) | TypeScript JSX | 10 | 0 | 4 | 14 |
| [frontend/src/pages/modules/assignments/submissions/index/SubmissionsList.tsx](/frontend/src/pages/modules/assignments/submissions/index/SubmissionsList.tsx) | TypeScript JSX | 204 | 0 | 18 | 222 |
| [frontend/src/pages/modules/assignments/submissions/show/SubmissionView.tsx](/frontend/src/pages/modules/assignments/submissions/show/SubmissionView.tsx) | TypeScript JSX | 161 | 0 | 17 | 178 |
| [frontend/src/pages/settings/Account.tsx](/frontend/src/pages/settings/Account.tsx) | TypeScript JSX | 158 | 0 | 20 | 178 |
| [frontend/src/pages/settings/Appearance.tsx](/frontend/src/pages/settings/Appearance.tsx) | TypeScript JSX | 162 | 0 | 12 | 174 |
| [frontend/src/pages/settings/Security.tsx](/frontend/src/pages/settings/Security.tsx) | TypeScript JSX | 106 | 0 | 14 | 120 |
| [frontend/src/pages/shared/CalendarPage.tsx](/frontend/src/pages/shared/CalendarPage.tsx) | TypeScript JSX | 16 | 0 | 2 | 18 |
| [frontend/src/pages/shared/HelpPage.tsx](/frontend/src/pages/shared/HelpPage.tsx) | TypeScript JSX | 108 | 0 | 15 | 123 |
| [frontend/src/pages/shared/status/Forbidden.tsx](/frontend/src/pages/shared/status/Forbidden.tsx) | TypeScript JSX | 34 | 0 | 4 | 38 |
| [frontend/src/pages/shared/status/NotFound.tsx](/frontend/src/pages/shared/status/NotFound.tsx) | TypeScript JSX | 34 | 0 | 4 | 38 |
| [frontend/src/pages/shared/status/ServerError.tsx](/frontend/src/pages/shared/status/ServerError.tsx) | TypeScript JSX | 34 | 0 | 3 | 37 |
| [frontend/src/pages/shared/status/Unauthorized.tsx](/frontend/src/pages/shared/status/Unauthorized.tsx) | TypeScript JSX | 34 | 0 | 4 | 38 |
| [frontend/src/pages/shared/status/UnderConstruction.tsx](/frontend/src/pages/shared/status/UnderConstruction.tsx) | TypeScript JSX | 37 | 0 | 6 | 43 |
| [frontend/src/pages/users/UserView.tsx](/frontend/src/pages/users/UserView.tsx) | TypeScript JSX | 191 | 0 | 19 | 210 |
| [frontend/src/pages/users/UsersList.tsx](/frontend/src/pages/users/UsersList.tsx) | TypeScript JSX | 262 | 0 | 20 | 282 |
| [frontend/src/routes/ProtectedRoute.tsx](/frontend/src/routes/ProtectedRoute.tsx) | TypeScript JSX | 16 | 9 | 7 | 32 |
| [frontend/src/services/auth/get.ts](/frontend/src/services/auth/get.ts) | TypeScript | 44 | 0 | 11 | 55 |
| [frontend/src/services/auth/index.ts](/frontend/src/services/auth/index.ts) | TypeScript | 2 | 0 | 0 | 2 |
| [frontend/src/services/auth/post.ts](/frontend/src/services/auth/post.ts) | TypeScript | 60 | 0 | 7 | 67 |
| [frontend/src/services/modules/assignments/config/get.ts](/frontend/src/services/modules/assignments/config/get.ts) | TypeScript | 8 | 0 | 1 | 9 |
| [frontend/src/services/modules/assignments/config/index.ts](/frontend/src/services/modules/assignments/config/index.ts) | TypeScript | 2 | 0 | 0 | 2 |
| [frontend/src/services/modules/assignments/config/post.ts](/frontend/src/services/modules/assignments/config/post.ts) | TypeScript | 15 | 0 | 2 | 17 |
| [frontend/src/services/modules/assignments/delete.ts](/frontend/src/services/modules/assignments/delete.ts) | TypeScript | 29 | 0 | 3 | 32 |
| [frontend/src/services/modules/assignments/get.ts](/frontend/src/services/modules/assignments/get.ts) | TypeScript | 60 | 0 | 7 | 67 |
| [frontend/src/services/modules/assignments/index.ts](/frontend/src/services/modules/assignments/index.ts) | TypeScript | 4 | 0 | 0 | 4 |
| [frontend/src/services/modules/assignments/mark-allocator/get.ts](/frontend/src/services/modules/assignments/mark-allocator/get.ts) | TypeScript | 8 | 0 | 2 | 10 |
| [frontend/src/services/modules/assignments/mark-allocator/index.ts](/frontend/src/services/modules/assignments/mark-allocator/index.ts) | TypeScript | 3 | 0 | 0 | 3 |
| [frontend/src/services/modules/assignments/mark-allocator/post.ts](/frontend/src/services/modules/assignments/mark-allocator/post.ts) | TypeScript | 10 | 0 | 1 | 11 |
| [frontend/src/services/modules/assignments/mark-allocator/put.ts](/frontend/src/services/modules/assignments/mark-allocator/put.ts) | TypeScript | 14 | 0 | 1 | 15 |
| [frontend/src/services/modules/assignments/memo-output/get.ts](/frontend/src/services/modules/assignments/memo-output/get.ts) | TypeScript | 8 | 0 | 3 | 11 |
| [frontend/src/services/modules/assignments/memo-output/index.ts](/frontend/src/services/modules/assignments/memo-output/index.ts) | TypeScript | 2 | 0 | 0 | 2 |
| [frontend/src/services/modules/assignments/memo-output/post.ts](/frontend/src/services/modules/assignments/memo-output/post.ts) | TypeScript | 10 | 0 | 1 | 11 |
| [frontend/src/services/modules/assignments/post.ts](/frontend/src/services/modules/assignments/post.ts) | TypeScript | 25 | 0 | 4 | 29 |
| [frontend/src/services/modules/assignments/put.ts](/frontend/src/services/modules/assignments/put.ts) | TypeScript | 43 | 0 | 5 | 48 |
| [frontend/src/services/modules/assignments/submissions/get.ts](/frontend/src/services/modules/assignments/submissions/get.ts) | TypeScript | 21 | 0 | 4 | 25 |
| [frontend/src/services/modules/assignments/submissions/index.ts](/frontend/src/services/modules/assignments/submissions/index.ts) | TypeScript | 1 | 0 | 0 | 1 |
| [frontend/src/services/modules/assignments/submissions/post.ts](/frontend/src/services/modules/assignments/submissions/post.ts) | TypeScript | 15 | 0 | 3 | 18 |
| [frontend/src/services/modules/assignments/tasks/delete.ts](/frontend/src/services/modules/assignments/tasks/delete.ts) | TypeScript | 11 | 0 | 1 | 12 |
| [frontend/src/services/modules/assignments/tasks/get.ts](/frontend/src/services/modules/assignments/tasks/get.ts) | TypeScript | 15 | 0 | 3 | 18 |
| [frontend/src/services/modules/assignments/tasks/index.ts](/frontend/src/services/modules/assignments/tasks/index.ts) | TypeScript | 4 | 0 | 0 | 4 |
| [frontend/src/services/modules/assignments/tasks/post.ts](/frontend/src/services/modules/assignments/tasks/post.ts) | TypeScript | 12 | 0 | 1 | 13 |
| [frontend/src/services/modules/assignments/tasks/put.ts](/frontend/src/services/modules/assignments/tasks/put.ts) | TypeScript | 18 | 0 | 1 | 19 |
| [frontend/src/services/modules/delete.ts](/frontend/src/services/modules/delete.ts) | TypeScript | 7 | 0 | 1 | 8 |
| [frontend/src/services/modules/get.ts](/frontend/src/services/modules/get.ts) | TypeScript | 28 | 0 | 4 | 32 |
| [frontend/src/services/modules/index.ts](/frontend/src/services/modules/index.ts) | TypeScript | 4 | 0 | 0 | 4 |
| [frontend/src/services/modules/personnel/delete.ts](/frontend/src/services/modules/personnel/delete.ts) | TypeScript | 14 | 0 | 2 | 16 |
| [frontend/src/services/modules/personnel/get.ts](/frontend/src/services/modules/personnel/get.ts) | TypeScript | 22 | 0 | 4 | 26 |
| [frontend/src/services/modules/personnel/index.ts](/frontend/src/services/modules/personnel/index.ts) | TypeScript | 3 | 0 | 0 | 3 |
| [frontend/src/services/modules/personnel/post.ts](/frontend/src/services/modules/personnel/post.ts) | TypeScript | 14 | 0 | 2 | 16 |
| [frontend/src/services/modules/post.ts](/frontend/src/services/modules/post.ts) | TypeScript | 12 | 0 | 1 | 13 |
| [frontend/src/services/modules/put.ts](/frontend/src/services/modules/put.ts) | TypeScript | 11 | 0 | 1 | 12 |
| [frontend/src/services/users/delete.ts](/frontend/src/services/users/delete.ts) | TypeScript | 7 | 0 | 1 | 8 |
| [frontend/src/services/users/get.ts](/frontend/src/services/users/get.ts) | TypeScript | 23 | 0 | 3 | 26 |
| [frontend/src/services/users/index.ts](/frontend/src/services/users/index.ts) | TypeScript | 3 | 0 | 0 | 3 |
| [frontend/src/services/users/post.ts](/frontend/src/services/users/post.ts) | TypeScript | 23 | 2 | 3 | 28 |
| [frontend/src/services/users/put.ts](/frontend/src/services/users/put.ts) | TypeScript | 11 | 0 | 1 | 12 |
| [frontend/src/types/auth/index.ts](/frontend/src/types/auth/index.ts) | TypeScript | 2 | 0 | 1 | 3 |
| [frontend/src/types/auth/responses.ts](/frontend/src/types/auth/responses.ts) | TypeScript | 13 | 6 | 6 | 25 |
| [frontend/src/types/auth/shared.ts](/frontend/src/types/auth/shared.ts) | TypeScript | 6 | 0 | 2 | 8 |
| [frontend/src/types/common/index.ts](/frontend/src/types/common/index.ts) | TypeScript | 36 | 8 | 10 | 54 |
| [frontend/src/types/modules/assignments/config/index.ts](/frontend/src/types/modules/assignments/config/index.ts) | TypeScript | 1 | 0 | 0 | 1 |
| [frontend/src/types/modules/assignments/config/shared.ts](/frontend/src/types/modules/assignments/config/shared.ts) | TypeScript | 28 | 48 | 15 | 91 |
| [frontend/src/types/modules/assignments/index.ts](/frontend/src/types/modules/assignments/index.ts) | TypeScript | 3 | 0 | 0 | 3 |
| [frontend/src/types/modules/assignments/mark-allocator/index.ts](/frontend/src/types/modules/assignments/mark-allocator/index.ts) | TypeScript | 2 | 0 | 0 | 2 |
| [frontend/src/types/modules/assignments/mark-allocator/requests.ts](/frontend/src/types/modules/assignments/mark-allocator/requests.ts) | TypeScript | 2 | 3 | 4 | 9 |
| [frontend/src/types/modules/assignments/mark-allocator/responses.ts](/frontend/src/types/modules/assignments/mark-allocator/responses.ts) | TypeScript | 4 | 6 | 4 | 14 |
| [frontend/src/types/modules/assignments/mark-allocator/shared.ts](/frontend/src/types/modules/assignments/mark-allocator/shared.ts) | TypeScript | 12 | 0 | 2 | 14 |
| [frontend/src/types/modules/assignments/memo-output/index.ts](/frontend/src/types/modules/assignments/memo-output/index.ts) | TypeScript | 1 | 0 | 0 | 1 |
| [frontend/src/types/modules/assignments/memo-output/responses.ts](/frontend/src/types/modules/assignments/memo-output/responses.ts) | TypeScript | 3 | 3 | 4 | 10 |
| [frontend/src/types/modules/assignments/memo-output/shared.ts](/frontend/src/types/modules/assignments/memo-output/shared.ts) | TypeScript | 10 | 0 | 2 | 12 |
| [frontend/src/types/modules/assignments/requests.ts](/frontend/src/types/modules/assignments/requests.ts) | TypeScript | 21 | 13 | 11 | 45 |
| [frontend/src/types/modules/assignments/responses.ts](/frontend/src/types/modules/assignments/responses.ts) | TypeScript | 23 | 12 | 16 | 51 |
| [frontend/src/types/modules/assignments/shared.ts](/frontend/src/types/modules/assignments/shared.ts) | TypeScript | 47 | 0 | 9 | 56 |
| [frontend/src/types/modules/assignments/submissions/index.ts](/frontend/src/types/modules/assignments/submissions/index.ts) | TypeScript | 2 | 0 | 0 | 2 |
| [frontend/src/types/modules/assignments/submissions/responses.ts](/frontend/src/types/modules/assignments/submissions/responses.ts) | TypeScript | 7 | 6 | 6 | 19 |
| [frontend/src/types/modules/assignments/submissions/shared.ts](/frontend/src/types/modules/assignments/submissions/shared.ts) | TypeScript | 39 | 3 | 7 | 49 |
| [frontend/src/types/modules/assignments/tasks/index.ts](/frontend/src/types/modules/assignments/tasks/index.ts) | TypeScript | 3 | 0 | 0 | 3 |
| [frontend/src/types/modules/assignments/tasks/requests.ts](/frontend/src/types/modules/assignments/tasks/requests.ts) | TypeScript | 5 | 0 | 0 | 5 |
| [frontend/src/types/modules/assignments/tasks/responses.ts](/frontend/src/types/modules/assignments/tasks/responses.ts) | TypeScript | 14 | 12 | 10 | 36 |
| [frontend/src/types/modules/assignments/tasks/shared.ts](/frontend/src/types/modules/assignments/tasks/shared.ts) | TypeScript | 13 | 0 | 2 | 15 |
| [frontend/src/types/modules/index.ts](/frontend/src/types/modules/index.ts) | TypeScript | 3 | 0 | 0 | 3 |
| [frontend/src/types/modules/personnel/index.ts](/frontend/src/types/modules/personnel/index.ts) | TypeScript | 3 | 0 | 0 | 3 |
| [frontend/src/types/modules/personnel/requests.ts](/frontend/src/types/modules/personnel/requests.ts) | TypeScript | 19 | 9 | 8 | 36 |
| [frontend/src/types/modules/personnel/responses.ts](/frontend/src/types/modules/personnel/responses.ts) | TypeScript | 10 | 9 | 7 | 26 |
| [frontend/src/types/modules/personnel/shared.ts](/frontend/src/types/modules/personnel/shared.ts) | TypeScript | 6 | 0 | 0 | 6 |
| [frontend/src/types/modules/requests.ts](/frontend/src/types/modules/requests.ts) | TypeScript | 3 | 6 | 5 | 14 |
| [frontend/src/types/modules/responses.ts](/frontend/src/types/modules/responses.ts) | TypeScript | 27 | 12 | 12 | 51 |
| [frontend/src/types/modules/shared.ts](/frontend/src/types/modules/shared.ts) | TypeScript | 15 | 0 | 4 | 19 |
| [frontend/src/types/users/index.ts](/frontend/src/types/users/index.ts) | TypeScript | 3 | 0 | 0 | 3 |
| [frontend/src/types/users/requests.ts](/frontend/src/types/users/requests.ts) | TypeScript | 9 | 6 | 8 | 23 |
| [frontend/src/types/users/responses.ts](/frontend/src/types/users/responses.ts) | TypeScript | 14 | 9 | 10 | 33 |
| [frontend/src/types/users/shared.ts](/frontend/src/types/users/shared.ts) | TypeScript | 7 | 0 | 1 | 8 |
| [frontend/src/utils/EventBus.ts](/frontend/src/utils/EventBus.ts) | TypeScript | 19 | 44 | 6 | 69 |
| [frontend/src/utils/api/index.ts](/frontend/src/utils/api/index.ts) | TypeScript | 152 | 37 | 34 | 223 |
| [frontend/src/utils/authSession.ts](/frontend/src/utils/authSession.ts) | TypeScript | 49 | 0 | 9 | 58 |
| [frontend/src/utils/message.tsx](/frontend/src/utils/message.tsx) | TypeScript JSX | 43 | 0 | 8 | 51 |
| [frontend/src/vite-env.d.ts](/frontend/src/vite-env.d.ts) | TypeScript | 0 | 1 | 1 | 2 |
| [frontend/tailwind.config.ts](/frontend/tailwind.config.ts) | TypeScript | 11 | 1 | 3 | 15 |
| [frontend/tsconfig.app.json](/frontend/tsconfig.app.json) | JSON | 28 | 0 | 4 | 32 |
| [frontend/tsconfig.json](/frontend/tsconfig.json) | JSON with Comments | 11 | 0 | 1 | 12 |
| [frontend/tsconfig.node.json](/frontend/tsconfig.node.json) | JSON | 21 | 2 | 3 | 26 |
| [frontend/vite.config.ts](/frontend/vite.config.ts) | TypeScript | 20 | 0 | 3 | 23 |
| [tests/README.md](/tests/README.md) | Markdown | 45 | 0 | 16 | 61 |
| [tests/cypress.config.js](/tests/cypress.config.js) | JavaScript | 34 | 0 | 3 | 37 |
| [tests/cypress/e2e/auth/login.cy.js](/tests/cypress/e2e/auth/login.cy.js) | JavaScript | 56 | 2 | 15 | 73 |
| [tests/cypress/e2e/auth/signup.cy.js](/tests/cypress/e2e/auth/signup.cy.js) | JavaScript | 60 | 0 | 14 | 74 |
| [tests/cypress/e2e/roles/admin/assignments.cy.js](/tests/cypress/e2e/roles/admin/assignments.cy.js) | JavaScript | 204 | 11 | 30 | 245 |
| [tests/cypress/e2e/roles/admin/modules.cy.js](/tests/cypress/e2e/roles/admin/modules.cy.js) | JavaScript | 74 | 1 | 14 | 89 |
| [tests/cypress/e2e/roles/admin/personnel.cy.js](/tests/cypress/e2e/roles/admin/personnel.cy.js) | JavaScript | 93 | 4 | 24 | 121 |
| [tests/cypress/e2e/roles/admin/submissions.cy.js](/tests/cypress/e2e/roles/admin/submissions.cy.js) | JavaScript | 51 | 3 | 9 | 63 |
| [tests/cypress/e2e/roles/admin/users.cy.js](/tests/cypress/e2e/roles/admin/users.cy.js) | JavaScript | 26 | 2 | 9 | 37 |
| [tests/cypress/e2e/roles/lecturer/assignments.cy.js](/tests/cypress/e2e/roles/lecturer/assignments.cy.js) | JavaScript | 57 | 9 | 12 | 78 |
| [tests/cypress/e2e/roles/lecturer/modules.cy.js](/tests/cypress/e2e/roles/lecturer/modules.cy.js) | JavaScript | 71 | 5 | 15 | 91 |
| [tests/cypress/e2e/roles/lecturer/personnel.cy.js](/tests/cypress/e2e/roles/lecturer/personnel.cy.js) | JavaScript | 72 | 5 | 13 | 90 |
| [tests/cypress/e2e/roles/lecturer/routes.cy.js](/tests/cypress/e2e/roles/lecturer/routes.cy.js) | JavaScript | 11 | 0 | 2 | 13 |
| [tests/cypress/e2e/roles/lecturer/submissions.cy.js](/tests/cypress/e2e/roles/lecturer/submissions.cy.js) | JavaScript | 86 | 0 | 11 | 97 |
| [tests/cypress/e2e/roles/student/routes.cy.js](/tests/cypress/e2e/roles/student/routes.cy.js) | JavaScript | 63 | 4 | 11 | 78 |
| [tests/cypress/e2e/roles/tutor/assignments.cy.js](/tests/cypress/e2e/roles/tutor/assignments.cy.js) | JavaScript | 73 | 5 | 13 | 91 |
| [tests/cypress/e2e/roles/tutor/routes.cy.js](/tests/cypress/e2e/roles/tutor/routes.cy.js) | JavaScript | 60 | 5 | 11 | 76 |
| [tests/cypress/e2e/roles/tutor/submissions.cy.js](/tests/cypress/e2e/roles/tutor/submissions.cy.js) | JavaScript | 88 | 7 | 12 | 107 |
| [tests/cypress/fixtures/config.json](/tests/cypress/fixtures/config.json) | JSON | 9 | 0 | 0 | 9 |
| [tests/cypress/fixtures/users.json](/tests/cypress/fixtures/users.json) | JSON | 22 | 0 | 0 | 22 |
| [tests/cypress/support/commands/api/auth.js](/tests/cypress/support/commands/api/auth.js) | JavaScript | 100 | 32 | 14 | 146 |
| [tests/cypress/support/commands/api/index.js](/tests/cypress/support/commands/api/index.js) | JavaScript | 3 | 0 | 0 | 3 |
| [tests/cypress/support/commands/api/modules/assignments/files.js](/tests/cypress/support/commands/api/modules/assignments/files.js) | JavaScript | 59 | 21 | 9 | 89 |
| [tests/cypress/support/commands/api/modules/assignments/index.js](/tests/cypress/support/commands/api/modules/assignments/index.js) | JavaScript | 243 | 88 | 27 | 358 |
| [tests/cypress/support/commands/api/modules/assignments/mark\_allocator.js](/tests/cypress/support/commands/api/modules/assignments/mark_allocator.js) | JavaScript | 21 | 9 | 3 | 33 |
| [tests/cypress/support/commands/api/modules/assignments/memo\_output.js](/tests/cypress/support/commands/api/modules/assignments/memo_output.js) | JavaScript | 21 | 9 | 3 | 33 |
| [tests/cypress/support/commands/api/modules/assignments/submissions.js](/tests/cypress/support/commands/api/modules/assignments/submissions.js) | JavaScript | 28 | 10 | 4 | 42 |
| [tests/cypress/support/commands/api/modules/assignments/tasks.js](/tests/cypress/support/commands/api/modules/assignments/tasks.js) | JavaScript | 82 | 31 | 7 | 120 |
| [tests/cypress/support/commands/api/modules/index.js](/tests/cypress/support/commands/api/modules/index.js) | JavaScript | 73 | 13 | 8 | 94 |
| [tests/cypress/support/commands/api/modules/personnel.js](/tests/cypress/support/commands/api/modules/personnel.js) | JavaScript | 68 | 22 | 8 | 98 |
| [tests/cypress/support/commands/api/users.js](/tests/cypress/support/commands/api/users.js) | JavaScript | 91 | 19 | 10 | 120 |
| [tests/cypress/support/commands/index.js](/tests/cypress/support/commands/index.js) | JavaScript | 2 | 0 | 1 | 3 |
| [tests/cypress/support/commands/ui/assignments.js](/tests/cypress/support/commands/ui/assignments.js) | JavaScript | 97 | 49 | 33 | 179 |
| [tests/cypress/support/commands/ui/auth.js](/tests/cypress/support/commands/ui/auth.js) | JavaScript | 19 | 25 | 5 | 49 |
| [tests/cypress/support/commands/ui/index.js](/tests/cypress/support/commands/ui/index.js) | JavaScript | 6 | 0 | 0 | 6 |
| [tests/cypress/support/commands/ui/modules.js](/tests/cypress/support/commands/ui/modules.js) | JavaScript | 51 | 65 | 16 | 132 |
| [tests/cypress/support/commands/ui/personnel.js](/tests/cypress/support/commands/ui/personnel.js) | JavaScript | 32 | 17 | 16 | 65 |
| [tests/cypress/support/commands/ui/submissions.js](/tests/cypress/support/commands/ui/submissions.js) | JavaScript | 27 | 22 | 8 | 57 |
| [tests/cypress/support/commands/ui/users.js](/tests/cypress/support/commands/ui/users.js) | JavaScript | 32 | 0 | 9 | 41 |
| [tests/cypress/support/commands/utils/api.js](/tests/cypress/support/commands/utils/api.js) | JavaScript | 1 | 0 | 1 | 2 |
| [tests/cypress/support/commands/utils/auth.js](/tests/cypress/support/commands/utils/auth.js) | JavaScript | 30 | 28 | 6 | 64 |
| [tests/cypress/support/e2e.js](/tests/cypress/support/e2e.js) | JavaScript | 2 | 0 | 1 | 3 |
| [tests/cypress/support/index.js](/tests/cypress/support/index.js) | JavaScript | 1 | 0 | 0 | 1 |
| [tests/jsconfig.json](/tests/jsconfig.json) | JSON with Comments | 14 | 2 | 2 | 18 |
| [tests/k6/scenarios/submissionLoad/index.js](/tests/k6/scenarios/submissionLoad/index.js) | JavaScript | 26 | 1 | 7 | 34 |
| [tests/k6/scenarios/submissionLoad/phases/01\_createModule.js](/tests/k6/scenarios/submissionLoad/phases/01_createModule.js) | JavaScript | 20 | 0 | 7 | 27 |
| [tests/k6/scenarios/submissionLoad/phases/02\_setupAssignment.js](/tests/k6/scenarios/submissionLoad/phases/02_setupAssignment.js) | JavaScript | 65 | 9 | 12 | 86 |
| [tests/k6/scenarios/submissionLoad/phases/03\_registerUsers.js](/tests/k6/scenarios/submissionLoad/phases/03_registerUsers.js) | JavaScript | 41 | 4 | 9 | 54 |
| [tests/k6/scenarios/submissionLoad/phases/04\_enrollUsers.js](/tests/k6/scenarios/submissionLoad/phases/04_enrollUsers.js) | JavaScript | 14 | 4 | 3 | 21 |
| [tests/k6/scenarios/submissionLoad/phases/05\_uploadSubmission.js](/tests/k6/scenarios/submissionLoad/phases/05_uploadSubmission.js) | JavaScript | 24 | 0 | 6 | 30 |
| [tests/k6/shared/auth.js](/tests/k6/shared/auth.js) | JavaScript | 19 | 0 | 5 | 24 |
| [tests/k6/shared/config.js](/tests/k6/shared/config.js) | JavaScript | 2 | 0 | 0 | 2 |
| [tests/k6/shared/http.js](/tests/k6/shared/http.js) | JavaScript | 14 | 8 | 4 | 26 |
| [tests/k6/test\_files/config.json](/tests/k6/test_files/config.json) | JSON | 10 | 0 | 0 | 10 |
| [tests/package-lock.json](/tests/package-lock.json) | JSON | 5,529 | 0 | 1 | 5,530 |
| [tests/package.json](/tests/package.json) | JSON | 16 | 0 | 0 | 16 |
| [tests/webpack.config.js](/tests/webpack.config.js) | JavaScript | 7 | 0 | 2 | 9 |

[Summary](results.md) / Details / [Diff Summary](diff.md) / [Diff Details](diff-details.md)