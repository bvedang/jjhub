# RIFT

## MVP Specification v2.0

_JJ-Native Code Review with Git Compatibility_

_February 2026_

---

Git-compatible, not Git-limited. Ship the review UX first, prove it works, then own the hosting story.

**v2.0 change:** Architecture rewritten around TypeScript/Node.js. The product vision, data model, and contracts are unchanged. What changed is _how_ it gets built — optimizing for speed-to-validation over long-term performance. Rust remains the eventual target for the storage engine once the product is proven.

## 1. Product Vision

Rift is the code review platform built for jj. Instead of forcing jj users into GitHub's branch-based PR model, Rift treats revisions, changes, and stacks as first-class objects. The result is stacked review that actually works: you push a chain of revisions, reviewers see each one independently, you amend and restack freely, and nothing breaks.

Git users aren't shut out. They can clone, fetch, and (in later phases) push for review. But jj is the primary workflow, and the UX is designed around jj's model.

### 1.1 What the MVP Must Prove

**Stacked review works.** A team of ~5 can run a full loop: create a revision stack, review each revision independently, amend and restack, comments never disappear, merge, and archive. This is the core product.

**Mirror mode is viable.** A team can point Rift at their existing GitHub repo, use Rift's review UX, and merge results back to the origin. This removes the adoption barrier of moving repos.

**JJ push-to-review is smooth.** A jj user can host a repo on Rift, push revisions, and browse them on the web without friction.

**Git clone/fetch works.** Git users can clone and fetch from a Rift-hosted repo reliably.

## 2. MVP Non-Goals

These are real features in the roadmap but not required for MVP. We keep design hooks so they slot in cleanly later.

- Conflict dashboard and async conflict resolution
- Server-wide operation log, undo, and time-travel UI
- Built-in CI (Rift Actions) and external CI integrations
- Full-text search across repos
- Webhooks and event streaming
- Git push-for-review (`refs/for/main`) — moved to fast follow after MVP. Git users get clone/fetch only in v1.

## 3. Core Concepts

These terms must be consistent across the UI, API, CLI, and docs. If anyone on the team uses them differently, fix it immediately.

### 3.1 The Four Primitives

| Concept  | Definition                                                                                                                                                                    |
| -------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Revision | Immutable snapshot. The unit of history. Once created, a revision never changes.                                                                                              |
| Change   | Mutable reference to a revision. The change_id is stable across amends. This is what makes stacked review work — you amend a revision and the change keeps the same identity. |
| Stack    | Ordered chain of revisions submitted for review. Replaces the PR. A stack has a status: open, merged, or closed.                                                              |
| Bookmark | Optional named pointer, used for integration points like main. Not required for stacks to exist.                                                                              |

### 3.2 MVP Invariants

Every revision has: `revision_id`, `change_id`, `parents[]`, `tree_hash`, `delta_hash`, `author`, `timestamp`.

`delta_hash` = hash of `diff(parent, revision)`. This is the "content of the change" independent of base. Used for interdiff, "did this change change?" detection, and future selective approval reset. Not the same as `tree_hash`, which changes on restack even if the change's patch is identical.

A stack has an ordered list of revisions and a status: `open | merged | closed`.

Comments are anchored to `(change_id, revision_id, file_path, line_number)`.

Stacks are linear-only in MVP. No DAG stacks.

Squash merge is the only merge strategy in MVP.

**change_id authority:** for stacks created via `jj rift push`, client-supplied change_ids are authoritative (JJ produces them). For history imported via mirror sync, Rift generates internal change_ids for bookkeeping, but these are not used for stack identity.

## 4. Architecture

**v2.0 change:** The original spec called for a Rust backend. For MVP, the entire backend is TypeScript/Node.js. The reasoning: validate the product before optimizing the runtime. TypeScript lets us share types between backend and frontend, move faster on API iteration, and avoid the Rust learning curve blocking shipping. The architecture is designed so the storage-heavy pieces (packfile generation, diff engine) can be extracted to Rust later if performance demands it.

### 4.1 Two-Layer Architecture

| Layer    | Technology                    | Responsibility                                                                                         |
| -------- | ----------------------------- | ------------------------------------------------------------------------------------------------------ |
| Backend  | TypeScript (Node.js, Fastify) | Storage engine, platform logic (auth, stacks, reviews, comments, merge), REST API, Git compat endpoint |
| Frontend | Next.js (React)               | Web UI: repo browser, stack review, DAG visualizer, admin                                              |

**Why Fastify over Express:** Fastify has built-in schema validation via JSON Schema, better TypeScript support, and significantly better performance benchmarks. For an MVP that might need to handle packfile streaming and diff computation, the performance headroom matters.

**CLI:** TypeScript (compiled with `tsx` or bundled with `pkg`/`esbuild`). Shares types with the backend via a shared package. Talks to the backend via REST.

**Monorepo structure:**

```
rift/
├── packages/
│   ├── shared/          # Types, constants, validation schemas
│   │   └── src/
│   │       ├── types.ts       # Revision, Change, Stack, etc.
│   │       ├── errors.ts      # Error codes (Section A.3)
│   │       └── validation.ts  # Shared validation logic
│   ├── backend/         # Fastify server
│   │   └── src/
│   │       ├── server.ts
│   │       ├── modules/
│   │       │   ├── storage/    # Git object storage, S3 interaction
│   │       │   ├── push/       # Push validation, atomic push
│   │       │   ├── stacks/     # Stack CRUD, iteration mgmt
│   │       │   ├── review/     # Reviews, approvals, gating
│   │       │   ├── diff/       # Diff engine, interdiff
│   │       │   ├── git/        # Git smart HTTP (clone/fetch)
│   │       │   ├── mirror/     # Mirror sync, merge-back
│   │       │   └── auth/       # OAuth, tokens
│   │       └── db/
│   │           ├── schema.ts   # Drizzle ORM schema
│   │           └── migrations/
│   ├── frontend/        # Next.js app
│   │   └── src/
│   │       ├── app/           # App router pages
│   │       └── components/    # Review UI, diff viewer, DAG
│   └── cli/             # jj rift CLI
│       └── src/
│           ├── commands/
│           │   ├── auth.ts
│           │   └── push.ts
│           └── index.ts
├── package.json         # Workspace root
└── turbo.json           # Turborepo config
```

**Workspace tooling:** pnpm workspaces + Turborepo. This gives us shared types without publishing packages, parallel builds, and good caching.

### 4.2 Infrastructure

**Postgres:** users, repos, stacks, reviews, comments, permissions. Accessed via Drizzle ORM (type-safe, lightweight, good migration story).

**Redis:** diff cache, session cache. Invalidated on push. Use `ioredis`.

**S3/MinIO:** content-addressed blob storage for Git objects. Immutable. Only the storage module touches it. Use `@aws-sdk/client-s3`.

**Queue (optional for MVP):** BullMQ (Redis-backed) if mirror sync needs async processing. Otherwise, direct calls.

### 4.3 Storage Object Model

MVP uses Git's object model internally. This is the boring choice, and it's the right one — it means GitUploadPack works by streaming packfiles from real Git objects, not by synthesizing them on the fly.

**Objects:** Git blobs, trees, and commits. Stored content-addressed in S3/MinIO using their SHA-1 OID as the key.

`revision_id` = Git commit OID (hex string). This is the hash of the commit object.

`tree_hash` = Git tree OID for the revision's root tree.

`delta_hash` = SHA-256 of the unified diff output between parent tree and revision tree. Computed on push, stored in Postgres alongside the revision.

**Git object handling in TypeScript:** Use `isomorphic-git` for reading/writing Git objects (blobs, trees, commits) and computing OIDs. For packfile generation (GitUploadPack), use `isomorphic-git`'s packfile utilities or shell out to `git pack-objects` as a subprocess. The subprocess approach is simpler and correct — optimize later if it becomes a bottleneck.

**Diff computation:** Use `diff` (npm package) or `jsdiff` for line-level unified diffs. For the delta_hash computation, normalize the diff output per the canonicalization rules in Appendix A.1.

**Packfile generation fallback:** If `isomorphic-git`'s packfile support proves insufficient for GitUploadPack, the backend can shell out to `git upload-pack` against a bare repo reconstructed from S3 objects. This is the escape hatch. It's slower but guaranteed correct.

Postgres holds Rift-specific metadata only: stacks, reviews, comments, permissions, tokens. It does not store blob content.

### 4.4 Key Library Choices

| Concern                | Library            | Why                                                     |
| ---------------------- | ------------------ | ------------------------------------------------------- |
| HTTP framework         | Fastify            | Fast, schema validation, good TS support                |
| Database ORM           | Drizzle            | Type-safe, lightweight, SQL-close                       |
| Git objects            | isomorphic-git     | Pure JS Git implementation, reads/writes objects        |
| Diff engine            | jsdiff             | Mature line-level diff, sufficient for MVP              |
| S3 client              | @aws-sdk/client-s3 | Standard                                                |
| Redis                  | ioredis            | Mature, cluster support                                 |
| Auth (OAuth)           | @fastify/oauth2    | Fastify-native OAuth plugin                             |
| Hashing (SHA-256)      | Node.js `crypto`   | Built-in, no dependency needed                          |
| Encryption (tokens)    | libsodium-wrappers | secretbox for origin credential encryption              |
| Task queue (if needed) | BullMQ             | Redis-backed, good DX, TS-first                         |
| Monorepo               | pnpm + Turborepo   | Fast, good workspace support                            |
| Syntax highlighting    | Shiki (frontend)   | tree-sitter–based, runs in browser, great theme support |

**What about tree-sitter on the backend?** Not needed for MVP. Shiki handles syntax highlighting on the frontend. If we add AST-aware diffing post-MVP, we can add tree-sitter via `node-tree-sitter` or move that module to Rust.

### 4.5 Performance Boundaries and Rust Extraction Path

TypeScript is fast enough for MVP scale (~5 users, tens of repos). But know where the walls are:

**CPU-bound operations that may need Rust later:**

- Packfile generation for large repos (>100k objects)
- Diff computation on very large files (>10k lines)
- delta_hash canonicalization at high push volume

**The extraction plan:** Each of these is isolated in its own backend module. When performance matters, compile a Rust binary (via `napi-rs` for Node.js native addon, or as a subprocess). The module interface doesn't change — only the implementation behind it.

**For MVP:** Don't optimize. Shell out to `git` for packfile generation if `isomorphic-git` is slow. Use `jsdiff` for diffs. Measure before rewriting.

## 5. Dual Workflow Contract

### 5.1 JJ: Full-Fidelity Workflow (Primary)

JJ is the primary way to interact with Rift. The CLI supports two commands in MVP:

`jj rift auth` — store authentication token.

`jj rift push` — push revisions and create/update a stack. Stack identity is based on change IDs, not branches.

When a user runs `jj rift push`, the CLI calls the backend's push/validate endpoint, then push/revisions. The platform automatically creates or updates the stack.

#### Stack Identity and Update Rules

This is the contract that determines when a push creates a new stack vs updates an existing one. Getting this wrong causes confusion for both authors and reviewers.

**Rule 1: Local stack_id binding.** The first time an author pushes a chain of revisions, Rift creates a new stack and returns a stack_id. The CLI stores this stack_id locally (in jj's workspace metadata). Subsequent pushes from the same workspace send the stored stack_id.

**Rule 2: Server validates on update.** When a push includes a stack_id, the server checks that every change_id in the pushed chain either already belongs to that stack or is new. If a change_id belongs to a different open stack, the push is rejected with a clear error.

**Rule 3: One stack per change.** A change_id can belong to at most one open stack at a time. This is enforced via an `open_change_claims` table (DB-enforced claim per change_id). On push validation, the server claims every active change_id for the stack; if any change_id is already claimed by a different open stack, the push is rejected with `409 change_conflict`. When a stack is merged or closed, its claims are released.

**Rule 4: Dropped revisions.** If the author drops a revision from the middle of a stack (removes it locally and re-pushes), Rift marks that change as "dropped" in the stack. The revision and its comments remain in iteration history but are no longer part of the active review.

**Rule 5: Stack splitting.** If an author wants to split one stack into two, they close the original stack and push two new ones. MVP does not support automatic stack splitting.

#### Iteration Model

Every push to an existing stack bumps the stack-level iteration counter. This is coarser than per-revision tracking but much simpler to reason about.

Iteration is a monotonic counter on the stacks table: `current_iteration INTEGER DEFAULT 1`.

Each push increments `current_iteration` by 1.

`stack_revisions` records include the iteration they were part of. A revision that didn't change still gets a new row for the new iteration (same `revision_id`, new iteration number).

The interdiff UI uses the iteration counter to let reviewers pick any two iterations to compare.

#### Push Idempotency and Retry

Network failures during push are inevitable. The push protocol must be safe to retry.

**Atomic push:** PushRevisions is all-or-nothing. Either all revisions in the push are stored and the stack is updated, or nothing changes. No partial uploads. Implemented as a single Postgres transaction wrapping S3 uploads — if the DB transaction fails, S3 objects are orphaned (cleaned up by a periodic garbage collection job, not needed for MVP).

**Dedupe key:** `(stack_id, iteration, revision_id)`. If the server receives a push with the same stack_id and iteration number and all revision_ids match an already-completed push, it returns success without re-processing. This makes retries safe.

**Conflict detection:** if the server receives a push for stack_id with an iteration number that already exists but different revision_ids, it rejects with `409 Conflict`. This catches bugs in the CLI, not normal usage.

#### CLI Failure Modes

Since MVP enforces linear-only stacks, the CLI must reject pushes that don't fit. These are the exact failure cases and what the CLI tells the user.

| Condition                                        | CLI Behavior | User Message                                                                                                                     |
| ------------------------------------------------ | ------------ | -------------------------------------------------------------------------------------------------------------------------------- |
| Workspace has multiple heads                     | Refuse push  | "Multiple heads detected. Merge or rebase to a single head before pushing."                                                      |
| Selected range is not linear (has merge commits) | Refuse push  | "Non-linear history detected between \<bookmark\> and working copy. Rift stacks must be linear. Rebase to remove merge commits." |
| No reachable bookmark found                      | Refuse push  | "No bookmark found as stack base. Create a bookmark on your target (e.g., `jj bookmark set main`) or use `--base` to specify."   |
| Multiple bookmarks reachable (ambiguous base)    | Refuse push  | "Ambiguous stack base: found bookmarks \<list\>. Use `--base <bookmark>` to specify which one."                                  |
| Empty range (working copy is on the bookmark)    | Refuse push  | "Nothing to push. Working copy is already at \<bookmark\>."                                                                      |

The `--base` flag overrides automatic bookmark detection. It lets the author explicitly say "everything between here and that bookmark is my stack." This is the escape hatch for all ambiguous cases.

**CLI implementation note:** The CLI is a TypeScript package compiled to a standalone binary via `esbuild` + `pkg` (or distributed as an npm package). It imports types from `@rift/shared`. The CLI parses `jj log` output to detect the revision chain — it does not embed jj as a library. This keeps the CLI simple and avoids FFI complexity.

### 5.2 Git: Supported but Constrained

Git users can interact with Rift-hosted repos, but the experience is deliberately limited to preserve Rift's semantics.

#### MVP Git Support

| Operation           | MVP Status | Notes                                         |
| ------------------- | ---------- | --------------------------------------------- |
| git clone / fetch   | Supported  | Via GitUploadPack (smart HTTP)                |
| git push (direct)   | Blocked    | No direct pushes to `refs/heads/*`            |
| git push-for-review | Post-MVP   | `refs/for/<bookmark>` deferred to fast follow |

#### GitUploadPack Implementation

Smart HTTP Git protocol, handled by a dedicated Fastify route (`/repos/:owner/:name.git/info/refs` and `/repos/:owner/:name.git/git-upload-pack`).

**Option A (recommended for MVP):** Shell out to `git upload-pack` against a bare repo. On each request, the backend ensures a bare repo exists on disk (or in a tmpdir) with the correct objects and refs. This is the simplest correct implementation.

**Option B (post-MVP optimization):** Use `isomorphic-git` to generate packfiles in-process from S3 objects. Avoids disk I/O but requires more code.

For MVP, Option A. Keep a local bare repo cache per repository, updated on push and mirror sync. The cache is reconstructable from S3 at any time.

#### Git Refs Exposed

`refs/heads/<bookmark>` for bookmarks (e.g., main).

`refs/rift/stacks/<stack_id>` (optional): points to head of a stack so Git users can checkout and review locally.

#### Git Authentication and Visibility

Private repos require authentication for Git operations. The rules are simple:

**Auth method:** HTTP basic auth. Username is the Rift username (or email). Password is the same token from `jj rift auth`. This works with standard Git credential helpers out of the box.

**Public repos:** clone/fetch without auth. Stack refs visible to everyone.

**Private repos:** clone/fetch requires Reader role or above. Bookmark refs and stack refs are visible to anyone with repo access.

**Mirror repos:** expose the same ref structure as native repos. Stack refs work identically.

#### Token Lifecycle

Since the same token is used for CLI auth, Git basic auth, and (separately) mirror mode origin credentials, the MVP needs basic token management.

**Token type:** long-lived personal access tokens. No automatic expiry in MVP. Users can optionally set an expiry date on creation.

**Revocation:** web UI has a Settings → Tokens page. Users can list active tokens and revoke any of them immediately. Revoked tokens are rejected on next use.

**Scopes:** MVP has two scopes: `read` (clone/fetch only) and `write` (push, create stacks, merge). Default is `write`. Readers who only need to clone can create a read-only token.

**Mirror origin credentials:** stored separately from user tokens. These are per-repo credentials (GitHub PAT or deploy key) managed by the repo Owner in the repo settings UI. Not the same as user tokens. Owner can rotate credentials in repo settings; rotation updates `encrypted_blob` and `rotated_at`.

**API endpoint:** `GET /v1/auth/tokens` (list), `POST /v1/auth/tokens` (create with optional expiry + scope), `DELETE /v1/auth/tokens/:id` (revoke).

#### Security Boundary

**User tokens:** stored as SHA-256 hashes in `auth_tokens.token_hash`. Plaintext is shown once at creation and never stored. Comparison is hash-to-hash. Use Node.js `crypto.createHash('sha256')`.

**Origin credentials:** encrypted at rest using `libsodium-wrappers` secretbox with a single server key (environment variable). Decryption happens only in the backend process, only when executing a mirror sync or merge push. Never exposed via API.

**MVP boundary:** libsodium secretbox with a single server key is sufficient. KMS integration is a post-MVP hardening step.

## 6. Mirror Mode

This is the most important addition to the spec. Mirror mode lets teams try Rift's review experience without moving their repos. It collapses the adoption barrier from "migrate your entire workflow" to "add a review tool."

### 6.1 How It Works

**Connect:** Create a mirrored repo in Rift by providing a Git remote URL (e.g., a GitHub repo). Rift clones it and sets up periodic sync.

**Sync inbound:** Rift pulls from the origin on a configurable interval (default: every 60 seconds). Webhook-triggered sync is deferred to post-MVP. New commits on tracked bookmarks (e.g., main) appear in Rift automatically.

**Review in Rift:** Authors push stacks to Rift using `jj rift push`. Review happens entirely in Rift's UI.

**Merge back:** When a stack is merged in Rift, the merged commit is pushed back to the origin's target bookmark. The origin stays the source of truth for CI, deployments, etc.

**Sync implementation:** Use `simple-git` (Node.js wrapper around the git CLI) for fetch and push operations against the origin. This is simpler and more reliable than implementing the Git protocol in pure JS. The backend spawns `git fetch` and `git push` as child processes with the decrypted origin credentials.

### 6.2 Data Model Addition

Add a `source` field to the repositories table:

```
source:       native | mirror
origin_url:   text (nullable, set for mirror repos)
sync_interval: integer (seconds, default 60)
last_synced:  timestamp
```

### 6.3 Mirror Mode Constraints (MVP)

Mirror repos are read-only on the Git side. Rift never force-pushes to the origin — only fast-forward merges.

If the origin has diverged (someone pushed directly to GitHub), Rift detects the conflict on sync and flags it in the UI. Resolution is manual in MVP.

Mirror mode requires a personal access token or deploy key for the origin. Stored encrypted.

Sync is polling-only in MVP. Webhook-triggered sync is post-MVP (requires the webhooks/events infrastructure).

**Tracked refs:** MVP syncs the default bookmark only (typically main). Repo owners can configure an allowlist of additional bookmarks to track in repo settings. Tags and all other refs are ignored until post-MVP.

**Polling implementation:** Use `node-cron` or a simple `setInterval` per repo. For MVP scale (tens of repos), this is fine. If it needs to scale, move to BullMQ repeatable jobs.

### 6.4 Mirror Merge Algorithm

The most dangerous moment in mirror mode is merge time. The origin may have moved since the stack was created. Here's the exact sequence:

1. **Author clicks merge.** The UI sends `POST /v1/repos/:owner/:name/stacks/:id/merge`.
2. **Rift re-fetches origin.** Before doing anything, Rift pulls the latest state of the target bookmark from the origin (via `simple-git` fetch). This ensures we're working with the real tip, not a stale cache.
3. **Compute squash commit.** Rift creates the squash-merge commit with parent = latest origin tip (not the tip from when the stack was created). The commit message combines the stack's revision descriptions. Use `isomorphic-git` to create the commit object.
4. **Push to origin.** Rift attempts a fast-forward push of the merge commit to the origin's target bookmark (via `simple-git` push).
5. **If push succeeds:** stack status moves to `merged`. Rift's local bookmark advances. Done.
6. **If push fails (non-fast-forward):** this means the origin moved between step 2 and step 4. Rift marks the stack as "rebase required" and blocks the merge. The author must pull the latest main, restack locally, and push a new iteration before trying again.
7. **If the squash commit cannot be computed cleanly** (the stack's changes conflict with new commits on the target bookmark), Rift also marks the stack as "rebase required." The UI shows the conflicting files and instructs the author to rebase locally and re-push. Rift does not attempt automatic conflict resolution in MVP — conflict tooling is a non-goal.

Both failure cases set the `stacks.blocked_reason` field (`rebase_required` or `conflicts`). The `blocked_reason` is cleared automatically when the author pushes a new iteration. While blocked, the merge button is disabled with an explanation.

For native repos (not mirrored), the merge is simpler: Rift is the source of truth, so there's no race. The squash commit's parent is always the current tip of the target bookmark in Rift's own storage.

#### Open Stacks and Base Tracking

Open stacks are pinned to a base revision for display purposes. When a stack is created, Rift records the current tip of the target bookmark as the stack's `base_revision_id`. All diff views show the stack's changes relative to this pinned base.

If the target bookmark advances (because another stack merged or the origin synced), the UI shows a "base behind by N commits" indicator on the stack overview. The diffs don't change — they still show the original patch.

**On merge attempt:** Rift uses the current tip (not the pinned base) as the squash commit's parent. If the squash can't be computed cleanly against the current tip, it sets `blocked_reason`.

**On new iteration push:** the `base_revision_id` is updated to the current tip of the target bookmark. This effectively "rebases the display" onto the latest main.

**Why pinned (not virtual rebase):** virtual rebase requires computing "what would these patches look like applied to the new base" on every page load. That's a conflict-resolution problem, which is a non-goal. Pinned base is simple, predictable, and correct.

#### Inbound Commits from Origin

When mirror sync pulls new commits from the origin (e.g., someone pushed directly to GitHub), those commits only advance the tracked bookmarks and update the revision graph. They do not create stacks. Stacks are only created by explicit `jj rift push`. This is important — without this rule, you'd need "PR inference" logic to guess which commits belong together, which is exactly the kind of complexity MVP avoids.

### 6.5 Mirror Sync State Machine

Mirror repos have a sync state that tracks the health of the connection to the origin.

| State      | Trigger                                                                          | Behavior                                                                                                                                                   |
| ---------- | -------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| healthy    | Normal sync: origin fetch succeeds and tracked refs fast-forward cleanly         | Bookmarks advance, revision graph updated. All operations work normally.                                                                                   |
| diverged   | Sync detects non-fast-forward on a tracked ref (e.g., force-push to origin main) | Set `blocked_reason=origin_diverged` on all open stacks targeting that ref. New stack pushes still allowed. Merges blocked until resolved.                 |
| sync_error | Network failure, auth failure, or origin unreachable                             | Log error, retry on next interval. After 3 consecutive failures, show banner in repo UI. Stacks not blocked (stale data is better than blocked workflows). |

**Clearing diverged state:** the repo Owner goes to repo settings and clicks "Accept origin as truth." Rift resets its internal bookmark to match the origin's current ref, discarding any local-only history on that bookmark. Open stacks that targeted the diverged ref have their `blocked_reason` cleared, and authors are notified that they should rebase against the new base.

**UI for diverged state:** a banner at the top of the repo page showing "Origin has diverged on \<bookmark\>. Merges are blocked. Owner action required." Link to repo settings.

## 7. Full Lifecycle Walkthrough

This section walks through every major workflow end-to-end. If you can't picture exactly what happens at each step, the spec isn't clear enough. These flows are the mental model for the entire product.

### 7.1 Onboarding and Repo Setup

#### Native Repo (Rift-Hosted)

1. **Sign up:** OAuth login via GitHub or Google.
2. **Create repo:** `POST /v1/repos` with a name. Rift creates the repo with a default main bookmark and an empty root revision.
3. **Clone locally:** Run `jj git clone https://rift/<owner>/<repo>.git`. Author now has a local jj workspace.
4. **Configure auth:** Run `jj rift auth`. Stores a token locally.

#### Mirror Repo (Existing GitHub/GitLab)

1. **Sign up:** Same OAuth flow.
2. **Create mirror:** `POST /v1/repos` with `source: mirror` and `origin_url` pointing to the GitHub repo. Provide access credentials.
3. **Initial sync:** Rift clones the origin (via `simple-git`). All existing history appears in Rift's revision graph.
4. **Clone from Rift:** Author clones from Rift (not GitHub) to get the jj-native experience. `jj git clone https://rift/<owner>/<repo>.git`
5. Team keeps using GitHub for CI/deploys. Rift is the review layer on top.

### 7.2 Author Flow: Create a Stack

This is the core workflow. A developer has a feature or fix that involves multiple logical changes.

1. **Write code in jj.** The author creates a chain of revisions locally. Each revision is a single logical change: maybe revision A adds a new data model, revision B adds the API endpoint, and revision C adds tests.
2. **Push to Rift.** The author runs `jj rift push`. The CLI detects the chain of revisions between the working copy and the nearest bookmark (e.g., main). It sends them to Rift as a stack.
3. **Rift creates the stack.** On the server, Rift stores each revision with the client-supplied change_ids (JJ is authoritative), links them in order, creates a stack with status `open` and records the current target bookmark tip as the `base_revision_id`. The stack appears in the web UI immediately.
4. **Request review.** The author opens the stack in the web UI and requests review from teammates. Each reviewer gets a link to the stack.

### 7.3 Reviewer Flow: Review a Stack

Reviewers interact with stacks, not branches. Here's what they see:

**Stack overview.** A vertical list of revisions in order. Each shows its title (first line of description), author, diff stats (+/- lines), and review state (Pending, Approved, Changes Requested). Think of it like a table of contents for the changeset.

**Per-revision diff.** Click any revision to see a side-by-side diff scoped to just that revision's changes. This is the core value prop — you review each logical change in isolation, not a 500-line mega-diff.

**Full-stack diff.** Toggle to see the combined diff of the entire stack against the base. Useful for understanding the overall impact.

**Leave comments.** Inline comments on specific lines of specific revisions. Comments are anchored to the revision they were left on.

**Approve or request changes.** Per-revision review state. A reviewer can approve revision A, request changes on revision B, and skip revision C.

### 7.4 Author Flow: Amend and Restack

The author gets feedback. Revision B needs changes. Here's what happens:

1. **Amend locally.** The author uses jj to amend revision B. Since jj tracks changes by change_id, the amended revision gets a new revision_id but keeps the same change_id.
2. **Restack.** Revision C (which was on top of B) automatically rebases onto the new B. This is jj doing what jj does.
3. **Push again.** `jj rift push` sends the updated stack. Rift sees the same change_ids and updates the existing stack rather than creating a new one.
4. **Rift shows iteration history.** The stack now has two iterations. Reviewers can see what changed between iteration 1 and iteration 2 — the interdiff. This is the killer feature. GitHub can't do this. Old comments stay attached to the old revision and are visible in a "previous iterations" view.
5. **Reviewer re-reviews.** The reviewer looks at the interdiff for revision B (what changed since their last review), confirms the fix, and approves.

### 7.5 Merge

1. **All revisions approved.** Every revision in the stack is marked Approved.
2. **Author or Owner merges.** Writers can merge their own stacks; Owners can merge any stack. Clicks merge in the UI. Rift squash-merges the stack onto the target bookmark (e.g., main).
3. **Stack archived.** The stack moves to status `merged`. It's still browsable for history but no longer active.
4. **Mirror sync (if applicable).** For mirrored repos, Rift pushes the merged commit back to the GitHub origin. CI picks it up from there.

### 7.6 Git User Flow (MVP)

A Git user on the team who hasn't adopted jj yet.

1. **Clone from Rift.** `git clone https://rift/<owner>/<repo>.git` works as expected.
2. **Pull updates.** `git fetch` / `git pull` picks up merged changes on main.
3. **Review in web UI.** The Git user reviews stacks in Rift's web interface, same as everyone else. They can leave comments. If they have Writer/Owner role, they can also approve and request changes.
4. **Submit changes.** In MVP, Git users who want to submit changes for review need to either use jj locally or wait for the post-MVP `refs/for/` push path. They can also push to the GitHub origin directly (for mirror repos), which syncs into Rift.

## 8. Stack Review UX (Detailed)

The review UX is the product. If this isn't great, nothing else matters. Here's what MVP must ship.

### 8.1 Stack Overview Page

**Vertical revision list.** Revisions in stack order (bottom = base, top = latest). Each row shows: revision title, author avatar, diff stats (+N / -N), review state badge.

**Stack metadata.** Target bookmark, author, creation date, current iteration count, overall status.

**Reviewer sidebar.** Requested reviewers, their per-revision approval state.

### 8.2 Per-Revision Diff View

Side-by-side diff scoped to a single revision. This is the default view when you click a revision.

File tree panel showing changed files in this revision only.

Inline commenting. Click a line to leave a comment. Comments are anchored to this specific revision.

Syntax highlighting using Shiki (tree-sitter–based, runs in the browser).

### 8.3 Full-Stack Diff View

Combined diff of the entire stack against the target bookmark.

Toggle between this and per-revision view with a single click.

Useful for final review before merge.

### 8.4 Interdiff View (The Differentiator)

When a revision is amended and the stack is re-pushed, Rift shows a diff-of-diffs: what changed between iteration N and iteration N+1 for a given change. This lets reviewers see exactly what the author fixed, without re-reviewing the entire revision.

Interdiff is computed at the patch level, not the tree level. For each revision, its "change delta" is `diff(parent, revision)`. Interdiff compares two change deltas across iterations. This means restacking alone (where the base changes but the patch is identical) produces an empty interdiff — which is correct. Tree-to-tree comparison would show noise from unrelated base changes.

Available from the stack overview by selecting two iterations to compare.

Per-revision interdiff (what changed in this specific change's delta between iterations).

Full-stack interdiff (what changed across all changes' deltas between iterations).

Quick signal: if a revision's `delta_hash` is the same across two iterations, the interdiff is empty and the UI shows "unchanged."

### 8.5 Frontend Diff Component Architecture

The diff viewer is the most complex frontend component. Here's how it breaks down:

**Diff data flow:** Backend computes the diff and returns structured hunks (not raw unified diff text). The API returns JSON with file-level entries, each containing hunks with line-by-line additions/deletions/context. The frontend renders this into a side-by-side or unified view.

**API response shape for diffs:**

```typescript
interface DiffResponse {
  files: Array<{
    path: string;
    old_path?: string; // set if renamed
    status: "added" | "modified" | "deleted" | "renamed";
    hunks: Array<{
      old_start: number;
      old_lines: number;
      new_start: number;
      new_lines: number;
      lines: Array<{
        type: "add" | "delete" | "context";
        content: string;
        old_line?: number;
        new_line?: number;
      }>;
    }>;
    stats: { additions: number; deletions: number };
  }>;
}
```

**Syntax highlighting:** Apply Shiki highlighting per-file on the frontend. Highlight the full file content, then map highlighted tokens back to diff lines. This avoids re-highlighting per hunk.

**Comment anchoring in the UI:** Each line in the diff view has a `(file_path, line_number)` coordinate. Clicking a line opens a comment form that captures `(change_id, revision_id, file_path, line_number)`.

## 9. Review and Approval Rules

The review model is simple in MVP but needs explicit rules. Without them, you'll ship a merge button that nobody trusts.

### 9.1 Who Can Approve

Only users with role Writer or Owner can submit approvals or request changes.

Readers can comment but cannot change review state.

The stack author cannot approve their own revisions.

### 9.2 Merge Gating

Every revision in the stack requires at least one approval and zero outstanding "Changes Requested" votes.

If any revision has a "Changes Requested" state from any reviewer, the merge button is disabled.

The merge button shows a clear breakdown: which revisions are approved, which are pending, which are blocked.

### 9.3 Approval Reset on Iteration

When the author pushes a new iteration, what happens to existing approvals? This is a tricky UX tradeoff. Keeping approvals means reviewers might miss regressions. Resetting everything means reviewers re-review code that didn't change.

**MVP rule:** reset all approvals on any stack update. When a new iteration is pushed, all review states across all revisions reset to Pending. This is aggressive but unambiguous. Reviewers use the interdiff view to quickly re-confirm unchanged revisions.

This is the safe default. Post-MVP, once the interdiff infrastructure is mature and we can confidently detect "this revision didn't change at all" (same `delta_hash` for that change_id across iterations), we can add selective reset: only reset approvals for revisions whose delta actually changed. Note: `tree_hash` is not suitable for this comparison because it changes on restack even when the patch is identical.

### 9.4 Review State Machine (Per Revision Per Reviewer)

Review submission is an upsert. `POST /.../stacks/:id/reviews` overwrites the previous state for the same `(stack_id, iteration, reviewer_id, revision_id)`. Re-submitting the same state is idempotent (no-op).

| From              | Action          | To                | Trigger          |
| ----------------- | --------------- | ----------------- | ---------------- |
| (none)            | Request review  | Pending           | Author requests  |
| Pending           | Approve         | Approved          | Reviewer submits |
| Pending           | Request changes | Changes Requested | Reviewer submits |
| Approved          | New iteration   | Pending           | Author pushes    |
| Changes Requested | New iteration   | Pending           | Author pushes    |

### 9.5 Review Request Semantics

**Adding reviewers:** `POST /stacks/:id/reviewers` is idempotent. Requesting the same reviewer again updates `requested_at` but doesn't create duplicates.

**Removing reviewers:** not supported in MVP. If you added the wrong person, they can simply not review. Removal comes post-MVP.

**Notifications:** out of scope for MVP. The `stack_reviewers` table is the data model for future in-app notifications and email. For now, authors share stack links directly.

## 10. Diff Engine

Stacked review lives or dies on diff quality. The diff engine is core infrastructure, not an afterthought. But scope it honestly — AST-level structural diffing is a multi-month project on its own.

### 10.1 MVP Requirements

**Line-based diffing.** Use `jsdiff` (structuredPatch / createTwoFilesPatch) as the base algorithm. This is the foundation everything else builds on.

**Syntax highlighting.** Use Shiki on the frontend for display-time highlighting in the diff view. This is not the same as syntax-aware diffing — it's just coloring the output.

**Whitespace handling.** Whitespace-insensitive mode (toggle in UI). Default: show whitespace changes dimmed, not highlighted. Implement via `jsdiff`'s `ignoreWhitespace` option for the toggle; rendering is frontend-only.

**File-level rename detection.** Detect files that were moved or renamed within a revision using content-similarity heuristics (same approach as Git's `-M` flag). Use `isomorphic-git` or shell out to `git diff --find-renames` for this. Show as moves, not delete + create. If detection fails, fall back to showing delete + add — acceptable in MVP.

**Interdiff computation.** Given two iterations of the same change (same change_id, different revision_ids), compute what changed between them. Critical: interdiff must be patch-to-patch, not tree-to-tree. Define Δ_a = `diff(parent_a, revision_a)` and Δ_b = `diff(parent_b, revision_b)`. Interdiff = `diff(Δ_a, Δ_b)`. Tree-to-tree would show noise from base changes on restack.

### 10.2 Post-MVP

**Syntax-aware structural diffing.** Use tree-sitter ASTs to show logical moves (function moved, block extracted) rather than line-level noise. Hard to do well. Not required for MVP. Can be added via `node-tree-sitter` or extracted to a Rust module.

**Intra-file move detection.** Detect code that moved within a file and show it as a move rather than a delete + insert.

### 10.3 Caching

Diff results cached per `(revision_id_a, revision_id_b)` pair in Redis.

Interdiff results cached per `(change_id, iteration_a, iteration_b)`.

Cache invalidated only when revisions are garbage-collected (which doesn't happen in MVP).

## 11. Comment System (Phased)

Comment anchoring across rebases is the hardest UX problem in the spec. Rather than trying to solve it perfectly in MVP, we ship something simple and correct, then iterate.

### 11.1 Phase 1 (MVP): Per-Revision Comments

**Anchor format:** `(change_id, revision_id, file_path, line_number)`.

**On amend/restack:** Old comments stay attached to the old revision. They do not move. The UI shows them in a "Previous iterations" tab when viewing the updated revision.

**No silent dropping.** Every comment is always visible somewhere.

**Why this works:** Reviewers can still see what they said and compare it against the new iteration. It's not magical, but it's correct and predictable.

### 11.2 Phase 2 (Post-MVP): Smart Remapping

Add `context_fingerprint` (hash of N surrounding lines) to the anchor.

On amend, attempt to remap comments to the new revision using context matching.

If remapping fails, mark the comment as outdated/unanchored with a link to the original location.

If remapping succeeds, show the comment inline on the new revision with a "carried forward" badge.

## 12. Data Model

### 12.1 Drizzle Schema (MVP Minimum)

The schema uses Drizzle ORM for type safety. Here's the shape — Drizzle schema definitions map directly to these tables.

| Table                   | Key Fields                                                                                                                                                                                                                       |
| ----------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| users                   | id, email, display_name, oauth_provider, created_at                                                                                                                                                                              |
| auth_tokens             | id, user_id, token_hash (never store plaintext), scope (read\|write), expires_at (nullable), created_at, revoked_at (nullable), last_used_at                                                                                     |
| repositories            | id, owner_id, name, default_bookmark, source (native\|mirror), origin_url, sync_interval, sync_state (healthy\|diverged\|sync_error), last_synced                                                                                |
| repo_origin_credentials | id, repo_id, credential_type (pat\|deploy_key), encrypted_blob, created_at, rotated_at. One active credential per repo.                                                                                                          |
| user_repository_roles   | user_id, repo_id, role (Owner\|Writer\|Reader)                                                                                                                                                                                   |
| revisions               | revision_id (git commit OID), repo_id, change_id, tree_hash (git tree OID), delta_hash (SHA-256 of parent→revision diff), author, description, timestamp                                                                         |
| revision_parents        | child_id, parent_id (adjacency list)                                                                                                                                                                                             |
| stacks                  | id, repo_id, author_id, target_bookmark, base_revision_id, status (open\|merged\|closed), blocked_reason (nullable: rebase_required\|origin_diverged\|conflicts), current_iteration (integer, default 1), created_at, updated_at |
| stack_revisions         | stack_id, revision_id, change_id, order_index, iteration (integer), status (active\|dropped)                                                                                                                                     |
| open_change_claims      | change_id (PK), stack_id, claimed_at. Exists only for active changes in open stacks; released on merge/close.                                                                                                                    |
| stack_reviewers         | stack_id, reviewer_id, requested_by, requested_at                                                                                                                                                                                |
| reviews                 | id, stack_id, reviewer_id, revision_id, iteration (integer), state (pending\|approved\|changes_requested). Unique on (stack_id, iteration, reviewer_id, revision_id)                                                             |
| comments                | id, stack_id, author_id, change_id, revision_id, file_path, line_number, body, created_at                                                                                                                                        |
| merges                  | stack_id, merged_revision_id, merged_at, merged_by                                                                                                                                                                               |

**Drizzle example (stacks table):**

```typescript
import {
  pgTable,
  uuid,
  text,
  integer,
  timestamp,
  pgEnum,
} from "drizzle-orm/pg-core";

export const stackStatusEnum = pgEnum("stack_status", [
  "open",
  "merged",
  "closed",
]);
export const blockedReasonEnum = pgEnum("blocked_reason", [
  "rebase_required",
  "origin_diverged",
  "conflicts",
]);

export const stacks = pgTable("stacks", {
  id: uuid("id").defaultRandom().primaryKey(),
  repoId: uuid("repo_id")
    .notNull()
    .references(() => repositories.id),
  authorId: uuid("author_id")
    .notNull()
    .references(() => users.id),
  targetBookmark: text("target_bookmark").notNull(),
  baseRevisionId: text("base_revision_id").notNull(),
  status: stackStatusEnum("status").notNull().default("open"),
  blockedReason: blockedReasonEnum("blocked_reason"),
  currentIteration: integer("current_iteration").notNull().default(1),
  createdAt: timestamp("created_at").notNull().defaultNow(),
  updatedAt: timestamp("updated_at").notNull().defaultNow(),
});
```

## 13. APIs

### 13.1 REST API (Fastify Backend)

All routes are defined with Fastify's schema validation using JSON Schema (or TypeBox for type-safe schema definitions). Request/response types are generated from schemas and shared with the frontend via `@rift/shared`.

#### Auth

- `POST /v1/auth/login` — OAuth web login
- `POST /v1/auth/token` — CLI token exchange (device flow)
- `GET /v1/auth/tokens` — list active tokens
- `POST /v1/auth/tokens` — create token (optional expiry + scope)
- `DELETE /v1/auth/tokens/:id` — revoke token

#### Push (CLI)

- `POST /v1/repos/:owner/:name/push/validate` — validate push (returns iteration)
- `POST /v1/repos/:owner/:name/push/revisions` — push revisions (atomic, idempotent)

#### Repositories

- `POST /v1/repos` — create repo (native or mirror)
- `GET /v1/repos/:owner/:name` — repo metadata
- `GET /v1/repos/:owner/:name/revisions/:id` — single revision (includes delta_hash)
- `GET /v1/repos/:owner/:name/graph?limit=&cursor=` — DAG for visualization (paginated)

#### Repository Browser

- `GET /v1/repos/:owner/:name/revisions/:id/tree?path=` — list tree entries at path
- `GET /v1/repos/:owner/:name/revisions/:id/blob?path=` — get file contents
- `GET /v1/repos/:owner/:name/revisions/:id/diff?base=&context=` — diff between revision and base (parent or specified revision_id)

#### Stacks and Review

- `POST /v1/repos/:owner/:name/stacks` — create stack
- `GET /v1/repos/:owner/:name/stacks?status=&author=&reviewer=&limit=&cursor=` — list stacks (paginated)
- `GET /v1/repos/:owner/:name/stacks/:id` — stack detail with revisions, review state, blocked_reason, base_revision_id
- `POST /v1/repos/:owner/:name/stacks/:id/reviewers` — request review from users (idempotent)
- `POST /v1/repos/:owner/:name/stacks/:id/reviews` — submit review (upsert)
- `POST /v1/repos/:owner/:name/stacks/:id/merge` — merge stack (blocked if blocked_reason is set)
- `POST /v1/repos/:owner/:name/stacks/:id/comments` — create inline comment
- `GET /v1/repos/:owner/:name/stacks/:id/comments` — list comments (paginated)
- `GET /v1/repos/:owner/:name/stacks/:id/interdiff?from=&to=` — interdiff between iterations (from/to are iteration numbers)

### 13.2 Pagination Contract

All list endpoints use cursor-based pagination with the same contract:

**Cursor:** opaque base64 string encoding `(sort_key, id)`. Clients must not parse or construct cursors — they come from the `next_cursor` field in responses.

**Default ordering:** `updated_at DESC, id DESC` for stacks. `created_at DESC, id DESC` for comments.

`updated_at` for stacks is bumped on: new push/iteration, review submission, comment creation, reviewer request, blocked_reason changes, and merge/close.

**Default limit:** 25. Maximum: 100. Specified via `limit` query param.

**Response shape:** `{ items: [...], next_cursor: "..." | null }`. If `next_cursor` is null, there are no more results.

**Filters are stable across pages:** the cursor encodes the filter set. Changing a filter requires a new query without a cursor.

### 13.3 Internal Module Interfaces

These are TypeScript module interfaces inside the backend, not separate services. Defined as classes or functions with typed inputs/outputs.

```typescript
// packages/backend/src/modules/push/push.service.ts
interface PushService {
  validatePush(input: ValidatePushInput): Promise<ValidatePushResult>;
  pushRevisions(input: PushRevisionsInput): Promise<PushRevisionsResult>;
}

// packages/backend/src/modules/storage/revision.service.ts
interface RevisionService {
  getRevision(repoId: string, revisionId: string): Promise<Revision>;
  getAncestors(
    repoId: string,
    revisionId: string,
    limit: number,
  ): Promise<Revision[]>;
  getDescendants(
    repoId: string,
    revisionId: string,
    limit: number,
  ): Promise<Revision[]>;
}

// packages/backend/src/modules/storage/tree.service.ts
interface TreeService {
  getTree(
    repoId: string,
    revisionId: string,
    path: string,
  ): Promise<TreeEntry[]>;
  getBlob(repoId: string, revisionId: string, path: string): Promise<Buffer>;
  getDiff(
    repoId: string,
    baseId: string,
    headId: string,
  ): Promise<DiffResponse>;
}

// packages/backend/src/modules/git/git.service.ts
interface GitService {
  handleUploadPack(repoId: string, request: Request): Promise<Response>;
}

// packages/backend/src/modules/mirror/mirror.service.ts
interface MirrorService {
  sync(repoId: string): Promise<SyncResult>;
  pushMerge(repoId: string, stackId: string): Promise<MergeResult>;
}
```

These interfaces are the extraction boundary. When a module needs to move to Rust (via napi-rs), only the implementation changes — the interface stays the same.

## 14. Repository Browser

MVP includes a web-based repo browser. It's not the differentiator, but it needs to be competent.

**File tree:** browsable at any revision. Click through directories, view files.

**Revision graph:** DAG visualizer showing the full history. Highlight stacks in progress. Use a React component with SVG rendering (libraries like `react-flow` or custom SVG).

**File viewer:** syntax highlighting via Shiki. Line numbers. Blame view is post-MVP.

## 15. Merge Rules

**MVP strategy:** squash merge only. The entire stack is squashed into a single commit on the target bookmark.

**Merge permissions:** Owners can merge any stack. Writers can merge their own stacks (stacks they authored). Readers cannot merge.

**Merge gating:** every revision requires ≥1 approval and 0 "Changes Requested" votes (see Section 9 for full rules). CI gating comes post-MVP.

**Blocked stacks:** if `stacks.blocked_reason` is set (`rebase_required`, `origin_diverged`, or `conflicts`), merge is disabled regardless of approvals. The UI shows the reason and instructions. The `blocked_reason` clears automatically when the author pushes a new iteration.

**Post-merge:** stack status moves to `merged`. Stack is archived but still browsable.

**Mirror merge:** for mirrored repos, the merge follows the algorithm in Section 6.4 — re-fetch origin, compute squash against latest tip, fast-forward push, fail gracefully if origin diverged or conflicts arise.

**Native merge:** for native repos, squash commit parent is the current tip of the target bookmark in Rift's storage. No race condition possible.

## 16. Permissions

### 16.1 MVP Roles

| Role   | Capabilities                                                                       |
| ------ | ---------------------------------------------------------------------------------- |
| Owner  | Repo settings, manage roles, merge any stack, push, create stacks, comment, review |
| Writer | Push (jj), create stacks, merge own stacks, comment, review, request review        |
| Reader | Browse, comment, cannot approve/request changes, cannot push or merge              |

### 16.2 Push Protection

Protected bookmarks: no force pushes to main.

Since merge happens through stacks, direct push to main is off by default.

Full role ladder (Admin, Maintainer) comes post-MVP.

## 17. Acceptance Criteria

MVP is shippable when all of the following work end-to-end:

**JJ full loop:** host repo → `jj rift push` → browse file tree and DAG → review per revision with inline comments → amend and restack → push again → see iteration history and previous comments → approve → merge → stack archived.

**Mirror mode loop:** connect GitHub repo → Rift syncs history → `jj rift push` creates stack → review and merge in Rift → merged commit appears on GitHub main.

**Interdiff works:** after amending a revision and re-pushing, the UI shows a clear diff-of-diffs between the old and new iteration.

**Git clone works:** `git clone https://rift/<owner>/<repo>.git` succeeds against both native and mirrored repos.

**Comments never disappear:** after amend/restack, every comment is still accessible — either on the current iteration's revision or in the previous iterations view. Prior-iteration context is always preserved.

**Approval reset works:** after pushing a new iteration, all review states reset to Pending. The merge button is blocked until all revisions are re-approved. Reviewers can use the interdiff to quickly re-confirm unchanged revisions.

**Mirror merge handles races:** if the origin has advanced since the last sync, the merge either succeeds (fast-forward) or clearly tells the author to rebase. No silent failures, no corrupted state.

**CLI rejects bad states:** pushing from a workspace with multiple heads, non-linear history, or no reachable bookmark produces a clear, actionable error message. No silent corruption of stacks.

**Token revocation works:** revoking a token via the web UI immediately blocks all Git and CLI operations using that token.

## 18. Open Decisions

These need to be locked in before implementation begins.

**Iteration storage model.** Store full revision snapshots per iteration (simple, more storage) or diffs between iterations (compact, more complexity)? Recommended: full snapshots for MVP. Storage is cheap; debugging is expensive.

**OAuth scope for mirror mode.** What permissions does Rift need on the origin repo? Minimum: read access for sync, write access for push-back on merge. Needs to be tested against both GitHub and GitLab's token permission models.

**isomorphic-git vs git subprocess.** For the initial prototype, shell out to `git` for anything `isomorphic-git` can't handle cleanly. Measure which operations need native git and which can stay in pure JS. Lock in the boundary before building the GitUploadPack handler.

## 19. Post-MVP Roadmap (Ordered)

For context on what comes next, roughly in priority order:

1. Git push-for-review (`refs/for/<bookmark>` with Change-Id footer support)
2. Selective approval reset (only reset approvals for revisions whose delta_hash actually changed between iterations)
3. Smart comment remapping (context fingerprinting, automatic relocation on amend)
4. Webhook-triggered mirror sync (requires webhooks/events infrastructure)
5. CI gating (merge requires green checks, external CI integrations)
6. Virtual rebase for display (compute how patches would look on current base without actual rebase)
7. **Rust extraction for performance-critical paths** (packfile generation, diff engine, delta_hash computation via napi-rs)
8. Team-based review assignment (review groups, round-robin, code ownership)
9. Syntax-aware structural diffing (AST-level move detection using tree-sitter)
10. Conflict dashboard (visualize and resolve conflicts in-browser)
11. Rebase-merge strategy (alternative to squash)
12. Webhooks and event streaming
13. Full-text search across repos

## Appendix A: Implementation Contracts

This appendix provides the engineering-level contracts that close the gap between "what to build" and "how to build it." Every decision here is binding for MVP.

### A.1 ID and Hashing Model

```
revision_id:  Git commit OID (SHA-1 hex, 40 chars)
              e.g., "a1b2c3d4e5f6..."

change_id:    JJ change ID (hex string, from client)
              Authoritative: JJ produces it, server stores it
              For mirror-synced commits: server generates a UUID (crypto.randomUUID())

tree_hash:    Git tree OID (SHA-1 hex)
              Changes on restack even if patch is identical

delta_hash:   SHA-256 of canonical change delta representation (base-independent):
```

Compute raw delta = `diff(parent, revision)` (linear parent) and normalize:

- Use unified diff format with rename detection disabled (treat renames as delete+add for hashing).
- Ignore hunk headers (`@@ ... @@`), line numbers, and all context lines; include only added (`+`) and removed (`-`) lines.
- Normalize line endings to LF and encode as UTF-8.
- Order files lexicographically by path.
- For binary changes, include the literal string: `BINARY <old_blob_oid> <new_blob_oid>`.

`delta_hash = sha256(normalized_delta)` — computed via `crypto.createHash('sha256')`.

Stable across restacks if the change delta (added/removed lines) is identical.

Used for: interdiff, "unchanged" detection, cache keys.

```
stack_id:     UUID v4 (server-generated on first push via crypto.randomUUID())
              Stored locally by CLI in jj workspace metadata
```

### A.2 Diff and Interdiff Contracts

#### Per-Revision Diff

The diff for a single revision R is always computed as `diff(parent(R), R)` where parent is the linear parent in the stack. This is the "change delta" — what this revision actually changed.

**Implementation:** Fetch both trees from S3 (via `isomorphic-git` or git subprocess), compute unified diff using `jsdiff.structuredPatch()` per file. Return structured `DiffResponse` (see 8.5).

#### Full-Stack Diff

The combined diff of the entire stack against the pinned `base_revision_id`. Computed as `diff(base_revision, stack_tip)`. Used for the "overall impact" view before merge.

#### Interdiff (Patch-to-Patch)

For change C at iteration a and iteration b:

```
delta_a = diff(parent_a(C), C_a)
delta_b = diff(parent_b(C), C_b)
interdiff = diff(delta_a, delta_b)

NOT: diff(tree(C_a), tree(C_b))  // wrong: includes base changes
```

Quick path: if `delta_hash_a == delta_hash_b`, interdiff is empty. UI shows "unchanged" badge. Skip computation entirely.

### A.3 Standard Error Codes

| Code | Error Key            | When                                                   |
| ---- | -------------------- | ------------------------------------------------------ |
| 400  | invalid_request      | Malformed JSON, missing required fields                |
| 401  | unauthorized         | Missing or invalid token                               |
| 403  | forbidden            | Valid token but insufficient role for this action      |
| 404  | not_found            | Repo, stack, revision, or user does not exist          |
| 409  | change_conflict      | change_id belongs to a different open stack            |
| 409  | iteration_conflict   | Push retry with same iteration but different revisions |
| 409  | merge_blocked        | Stack has blocked_reason set, cannot merge             |
| 422  | non_linear_stack     | Pushed revisions don't form a linear chain             |
| 422  | self_approval        | Author tried to approve own revision                   |
| 422  | unapproved_revisions | Merge attempted but not all revisions approved         |
| 502  | origin_unreachable   | Mirror sync or merge push failed to reach origin       |

**Fastify error handling:** Use a custom error handler plugin that maps these to consistent JSON responses:

```typescript
// packages/shared/src/errors.ts
export class RiftError extends Error {
  constructor(
    public code: number,
    public errorKey: string,
    message: string,
  ) {
    super(message);
  }
}

// Usage in route handler:
throw new RiftError(
  409,
  "change_conflict",
  `change_id ${changeId} belongs to stack ${existingStackId}`,
);
```

### A.4 Key Database Constraints and Indexes

```sql
-- One open stack per change_id (DB-enforced)
-- Enforced via open_change_claims(change_id PRIMARY KEY)
-- On /v1/repos/:owner/:name/push/validate: claim active change_ids for
-- the stack; if claimed by another open stack -> 409 change_conflict
-- On merge/close: DELETE FROM open_change_claims WHERE stack_id = ...
-- (release claims)
-- On push update for same stack: existing claims are reused (idempotent)

-- Fast stack listing (repo home page)
CREATE INDEX idx_stacks_repo_status
  ON stacks(repo_id, status, updated_at DESC, id DESC);

-- Comments by stack (review page)
CREATE INDEX idx_comments_stack
  ON comments(stack_id, created_at DESC, id DESC);

-- Reviews by stack + iteration (approval check)
CREATE INDEX idx_reviews_stack_iteration
  ON reviews(stack_id, iteration, revision_id);

-- Review uniqueness
CREATE UNIQUE INDEX idx_reviews_unique
  ON reviews(stack_id, iteration, reviewer_id, revision_id);

-- Token lookup (auth check on every request)
CREATE UNIQUE INDEX idx_token_hash
  ON auth_tokens(token_hash) WHERE revoked_at IS NULL;
```

**Drizzle migration note:** Define these indexes in the Drizzle schema file. Run `drizzle-kit generate` to produce SQL migrations, then `drizzle-kit migrate` to apply.

### A.5 Push Protocol Sequence

```
Client (jj rift push)           Server (Fastify backend)

1. Detect stack boundary
   (revs between WC and base)

2. POST /v1/repos/:owner/:name/push/validate -------> validatePush():
   { stack_id?, change_ids[],     - check auth + role
     revision_ids[] }             - check one-stack-per-change
                                  - check linear chain
                 <-------------- { ok: true, iteration: N }

3. POST /v1/repos/:owner/:name/push/revisions ------> pushRevisions() (atomic):
   { stack_id, iteration: N,      - upload git objects to S3
     revisions: [                 - compute delta_hash per rev
       { rev_id, change_id,       - insert stack_revisions rows
         tree_hash, parent,       - increment current_iteration
         blob_data }              - reset all review states
     ] }                          - update base_revision_id
                                  - clear blocked_reason
                 <-------------- { stack_id, iteration: N,
                                   url: "..." }

4. Store stack_id locally
   (jj workspace metadata)
```

**Atomicity in TypeScript:** Wrap steps in `pushRevisions()` in a Drizzle transaction (`db.transaction(async (tx) => { ... })`). S3 uploads happen first (outside the transaction); if the DB transaction fails, orphaned S3 objects are harmless and can be cleaned up later.

**Retry safety:** if step 3 times out and the client retries with the same stack_id and iteration N, the server checks if that iteration already exists with matching revision_ids. If yes, returns success. If revision_ids differ, returns `409 iteration_conflict`.

## Appendix B: Development Setup

Quick-start for contributors:

```bash
# Prerequisites: Node.js 20+, pnpm 9+, Docker (for Postgres/Redis/MinIO)

# Clone and install
git clone https://github.com/vedang/rift.git
cd rift
pnpm install

# Start infrastructure
docker compose up -d  # Postgres, Redis, MinIO

# Run migrations
pnpm --filter @rift/backend db:migrate

# Start dev servers (Turborepo runs all packages in parallel)
pnpm dev

# Backend: http://localhost:3001
# Frontend: http://localhost:3000
```

**Docker Compose services:**

- `postgres:16` on port 5432
- `redis:7` on port 6379
- `minio` on port 9000 (S3-compatible)

**Environment variables:** Copy `.env.example` to `.env`. Key variables: `DATABASE_URL`, `REDIS_URL`, `S3_ENDPOINT`, `S3_BUCKET`, `ENCRYPTION_KEY` (for origin credentials), `GITHUB_CLIENT_ID`, `GITHUB_CLIENT_SECRET`.
