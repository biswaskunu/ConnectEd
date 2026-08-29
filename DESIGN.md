# Design



# Backend 

### Auth
| Method | Endpoint | Description | Auth |
|---|---|---|---|
| POST | `/auth/register` | Register (role selected at signup; non-admin starts unapproved) | No |
| POST | `/auth/login` | Login, returns access+refresh tokens | No |
| POST | `/auth/refresh` | Exchange refresh token for new pair | No |
| POST | `/auth/logout` | Revoke session | Yes |
| GET | `/users/me` | Current user profile | Yes |
| PATCH | `/users/me` | Update profile (role-specific fields) | Yes |

### Courses & Enrollment
| Method | Endpoint | Description | Auth |
|---|---|---|---|
| POST | `/courses` | Create course | Trainer |
| GET | `/courses` | List/browse courses | Yes |
| GET | `/courses/:id` | Course detail | Yes |
| POST | `/courses/:id/enroll` | Enroll in course | Trainee |
| GET | `/courses/:id/enrollments` | List enrolled trainees | Trainer/Admin |

### Assessments
| Method | Endpoint | Description | Auth |
|---|---|---|---|
| POST | `/courses/:id/questionnaires` | Create questionnaire + questions | Trainer |
| GET | `/questionnaires/:id` | Get questionnaire (questions, no answers, for trainee) | Yes |
| POST | `/questionnaires/:id/submit` | Submit answers, get scored | Trainee |
| GET | `/questionnaires/:id/results` | View all submissions | Trainer |

### Trainer Library
| Method | Endpoint | Description | Auth |
|---|---|---|---|
| POST | `/library` | Upload content metadata + URL | Trainer |
| GET | `/courses/:id/library` | List content for a course | Yes |

### Feedback
| Method | Endpoint | Description | Auth |
|---|---|---|---|
| POST | `/courses/:id/feedback` | Submit feedback | Trainee |
| GET | `/courses/:id/feedback` | View feedback | Trainer/Admin |

### Admin
| Method | Endpoint | Description | Auth |
|---|---|---|---|
| GET | `/admin/users/pending` | List unapproved users | Admin |
| POST | `/admin/users/:id/approve` | Approve user | Admin |
| PATCH | `/admin/users/:id/role` | Change user role | Admin |
| GET | `/admin/dashboard` | Aggregate stats (enrollments, assessments, participation) | Admin |
| POST | `/notifications` | Publish announcement/achievement/content notice | Admin |
| GET | `/notifications` | List published notifications (homepage feed) | Yes |

### Competency Mapping
| Method | Endpoint | Description | Auth |
|---|---|---|---|
| POST | `/competency-tags` | Create a tag | Admin |
| POST | `/trainers/:id/tags` | Assign tag to trainer | Trainer (self) / Admin |
| POST | `/courses/:id/tags` | Assign tag to course | Trainer/Admin |
| GET | `/courses/:id/suggested-trainers` | Tag-match suggested trainers | Trainer/Admin |

### Duels (optional)
| Method | Endpoint | Description | Auth |
|---|---|---|---|
| POST | `/duels` | Challenge a peer on a questionnaire | Trainee |
| POST | `/duels/:id/respond` | Accept/decline | Trainee |
| POST | `/duels/:id/submit` | Submit duel answers | Trainee |
| GET | `/duels/:id` | Get duel status/result | Trainee |

## 3. Status Code Convention
`201` creation, `200` reads/updates, `204` deletes, `401` missing/invalid auth, `403` authenticated-but-unauthorized, `404` missing resource, `422` invalid input, `500` genuine server failures only.





# Frontend (will be added by Animesh)
