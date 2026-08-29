# PRD — ConnectEd

## 1. Overview
ConnectEd is a digital capacity-building and learning management portal for the Ministry of Earth Sciences (MoES) / India Meteorological Department (IMED), built for Smart India Hackathon 2026. It centralizes organizational training, competency development, and knowledge sharing across three user roles: Trainee, Trainer, and Admin.

**Problem Statement ID:** SIH26075 | **Category:** Software | **Theme:** Smart Education

## 2. Problem Statement (verbatim)
Participants are invited to design and develop ConnectEd — a Digital Capacity Building and Learning Management Portal — to support organizational training, competency development, and knowledge sharing through a centralized web-based platform.

## 3. Goals
- Give every trainee a single place to build a professional profile, enroll in courses, learn, and get assessed
- Give every trainer a way to publish content, run assessments, and track trainee performance
- Give admins approval control, org-wide visibility (dashboards), and a publishing channel (announcements)
- Automatically surface the right trainer for a subject via competency mapping, replacing manual assignment
- (Stretch) Increase engagement via gamified peer assessment ("student duels")

## 5. User Roles & Core Needs

### Trainee
- Create/edit professional profile: qualifications, work experience, interests, skills, certificates
- Browse and enroll in courses
- Access learning resources (trainer-uploaded content)
- Attempt subject-wise MCQ assessments before deadlines
- Submit feedback on courses/training content
- (Stretch) Challenge peers to timed MCQ duels

### Trainer
- Manage own profile (bio, subject/competency tags)
- Create questionnaires (MCQ sets) with deadlines, tied to a course
- Monitor trainee participation and performance per course/questionnaire
- Upload recorded lectures, presentations, study materials to a trainer library visible to enrolled trainees

### Admin
- Approve/reject new user registrations
- Manage user roles
- View dashboards: courses, enrollments, certifications, assessments, participation statistics
- Publish notifications, announcements, achievements, and newly added learning content to the homepage
- Oversee competency mapping data (view/adjust trainer-subject tags if needed)

## 6. Functional Requirements (by priority)

### P0 — Must ship for a complete demo
1. Auth: signup, login, role-based access (Trainee/Trainer/Admin), admin approval gate before non-admin accounts are active
2. Trainee profile CRUD
3. Trainer profile CRUD
4. Course creation (Trainer) + browsing/enrollment (Trainee)
5. Questionnaire + MCQ creation (Trainer), with deadline
6. Assessment attempt + auto-scoring (Trainee)
7. Trainer library upload + trainee access (file metadata + URL, not full media handling)
8. Admin approval queue + role management
9. Admin dashboards: enrollment counts, assessment stats, participation stats (basic aggregate queries)
10. Feedback submission (Trainee → course)
11. Notifications/announcements (Admin publish → homepage display)
12. Competency mapping: tag-based trainer-subject matcher, at least a "suggested trainers for this subject" view

### P1 — Nice to have if time allows
13. Certificate generation/tracking on course completion
14. Trainer view of individual trainee performance breakdown
15. Search/filter on course catalog

### P2 — Stretch only
16. Student duels: peer-to-peer timed MCQ mode reusing existing question bank, live or async score comparison

## 7. Non-Functional Requirements
- **Security:** bcrypt password hashing, JWT access+refresh with SHA-256 hashed storage, role-based route guards
- **Scalability language for pitch:** architecture should not preclude scaling beyond MoES to other ministries (no MoES-specific hardcoding in schema/logic)
- **Accessibility:** responsive UI, usable across devices (per PS requirement) — not a WCAG audit, but no fixed-width desktop-only layouts
- **Reliability for demo:** every P0 feature must be demonstrably working end-to-end by the feature-freeze checkpoint (see Phases doc)


## 8. Success Criteria (for the hackathon itself)
- All P0 features functional and demoable live (not just described in slides)
- Clean live demo path with realistic seed data
- Competency mapping visibly demonstrated, not just claimed
- PPT round: pass first elimination (see separate pitch notes)
- Grand Finale (if selected): 36-hour build executed against the Phases doc without major scope renegotiation mid-event
