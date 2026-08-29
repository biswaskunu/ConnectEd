# Phases

### v0.1 — Schema & Contract Lock
- Finalize DB schema (start from `DESIGN.md` draft, challenge every table before committing)
- Write the API contract (endpoint list, request/response shapes, auth requirements) — this unblocks frontend from hour 0
- Axum project skeleton: router structure, `AuthenticatedUser` extractor, error-handling convention, SQLx connection pool
- Frontend: component library decision, low-fi wireframes for all three dashboards, seed content drafted (realistic course names, questions, bios — not placeholder text)

### v0.2 — Auth Working End-to-End
- Register/login/refresh/logout implemented and tested
- Role enum + approval gate working
- Frontend auth screens wired to real backend (not mocked)

### v0.3 — PPT Round Ready
- Pitch deck complete (see separate pitch notes)
- Architecture diagram exported for the deck
- This is the checkpoint gating whether you even reach the 36-hour build — treat it as a hard milestone

### v0.4: Foundation
- Schema migrated, seed data loaded
- Auth flows confirmed working in the actual competition environment (not just local dev)
- API contract finalized/frozen — frontend unblocked to build against it
- Backend: role-guard middleware in place
- Frontend: app shell + navigation per role, dashboards scaffolded with mock data

### v0.5: Trainee & Trainer Core
- Backend: course CRUD, enrollment, questionnaire/MCQ CRUD, submission+scoring, trainer library upload
- Frontend: Trainee flows (profile, browse/enroll, take assessment, feedback), Trainer flows (profile, questionnaire builder, library upload) in progress

### v0.6: Admin & Dashboards
- Backend: approval queue, role management endpoints, dashboard aggregate queries, notifications/announcements
- Frontend: Admin flows (approval queue, role mgmt, dashboards/charts, publish announcements), continued polish on Trainee/Trainer screens

### v0.7: Competency Mapping
- Backend: tag-based matcher (trainer tags ↔ course/subject tags), "suggested trainers" endpoint
- Frontend: surface the match, even minimally — a visible "suggested trainer" badge/list is worth more in a demo than a hidden backend feature

## BUG CHECKS FOR A TIME

### v0.8: Integration, Polish, (Stretch) Duels
- Full integration pass: replace all frontend mock data with real API calls
- Bug-fixing buffer
- If green-lit: student duels — reuse question bank, add head-to-head/timed mode
- Visual polish pass on all screens — this is what judges see first

### v1.0: Freeze & Demo Prep
- Code freeze
- Full team dry run: verify every claim in the pitch deck matches what's actually running live
- Demo script rehearsed against the real, running app (not assumed)