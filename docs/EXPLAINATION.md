# RIFT

## What It Is and Why It Exists

_A guide for people who know Git but haven't used jj_

_February 2026_

---

Companion to the Rift MVP Specification (v1.6.1). Read this first. Read the spec when you're ready to build.

## The Problem

You've written a feature that touches three things: a database migration, an API endpoint, and a frontend component. On GitHub, you open one pull request with all three changes. The reviewer sees a 400-line diff and has to hold the whole thing in their head at once.

Or you try to break it up. You open three PRs, each depending on the one before it. Now you're managing branch dependencies by hand. When the first PR gets feedback, you rebase, force-push, and then update the second and third PRs manually. It's tedious, error-prone, and nobody enjoys it.

This is the stacked review problem. Everyone who's worked on a large codebase has hit it. The tooling just isn't there.

## Why GitHub PRs Don't Solve It

Pull requests are organized around branches. One branch, one PR, one diff. That model works fine for small, self-contained changes. It breaks down when your work is naturally a sequence of dependent changes.

Some teams use tools like Graphite or ghstack to simulate stacked PRs on top of GitHub. They work, mostly. But they're fighting the platform's assumptions. Every rebase is a force-push. Every update to an early PR requires cascading updates to later ones. The review history gets muddled because GitHub doesn't know these PRs are related.

The core issue: GitHub tracks branches, not changes. When you amend a commit and rebase the stack, GitHub sees entirely new commits. It can't tell you what actually changed between review rounds. Reviewers end up re-reading code they've already approved.

## What jj Changes

jj (Jujutsu) is a version control tool that's compatible with Git repositories but works differently under the hood. You don't need to understand all of jj to understand Rift, but two ideas matter.

### Idea 1: Changes Have Stable Identity

In Git, when you amend a commit, you get a new commit with a new hash. The old one and the new one have no connection. Git doesn't know they're the same logical change.

In jj, every change has a stable ID (called a change ID) that survives amends. You can rewrite a change ten times and the change ID stays the same. This means the system always knows "this is the same change, just updated."

This is the key insight that makes stacked review work. When you update one change in a stack, the tool knows exactly which one changed and which ones didn't. No guessing.

### Idea 2: Restacking Is Automatic

In Git, if you amend a commit in the middle of a branch, everything above it has a conflict with the old version. You have to manually rebase the later commits.

In jj, when you amend a change, all the changes that depend on it are automatically rebased. jj calls this "restacking." It's not a separate step — it just happens.

This is what makes stacked workflows practical. You're not managing a chain of branches. You're editing a stack of changes, and the tool keeps everything consistent.

## What Rift Is

Rift is a code review platform built around these ideas. Instead of pull requests organized by branches, Rift has stacks organized by changes.

Here's the translation table:

| Git / GitHub      | jj / Rift      | Why It Matters                                                    |
| ----------------- | -------------- | ----------------------------------------------------------------- |
| Commit            | Revision       | Same thing, different name. An immutable snapshot.                |
| (no equivalent)   | Change         | Stable identity across amends. This is what Git is missing.       |
| Pull Request      | Stack          | An ordered chain of changes for review. Can hold 1 or 20 changes. |
| Branch            | Bookmark       | A named pointer like main. Exists but isn't central to review.    |
| Force-push + hope | Push iteration | Each update is tracked. Reviewers see exactly what changed.       |

The most important row is the last one. When you push an updated stack to Rift, it doesn't throw away the history like a force-push. It creates a new iteration. Reviewers can compare any two iterations and see a diff-of-diffs: "what changed in this change since the last time I looked at it?"

Rift calls this the interdiff. It's the feature that makes stacked review actually usable for reviewers, not just authors.

## How It Works (The 5-Minute Version)

### 1. You Write Code

You're working in jj. You make three changes: A, B, and C. Each one builds on the one before it. A adds a data model. B adds an API endpoint that uses the model. C adds tests for the endpoint.

In Git terms, this is like three commits on a feature branch. The difference is that jj tracks each one by change ID, not commit hash.

### 2. You Push to Rift

You run `jj rift push`. The CLI detects the chain of changes between your working copy and main, and sends them to Rift as a stack.

Rift stores each change, links them in order, and creates a stack. It appears in the web UI immediately.

### 3. Reviewers See Each Change Separately

A reviewer opens the stack and sees three items: A, B, and C. They click A and see only the data model changes. They click B and see only the API endpoint. Each change is reviewed in isolation.

This is the core value. A 400-line mega-diff becomes three focused reviews of ~130 lines each.

### 4. You Get Feedback and Update

The reviewer wants a change to B. You amend B locally. jj automatically restacks C on top of the new B. You push again.

Rift sees the same change IDs and updates the existing stack. It bumps the iteration counter. The reviewer can now see exactly what changed in B between iteration 1 and iteration 2 — the interdiff. Change C might have a different commit hash (because B changed underneath it), but Rift knows C's actual content didn't change.

### 5. Approve and Merge

Each change gets approved individually. Once everything is approved, you merge. Rift squashes the stack into a single commit on main. The stack is archived but still browsable.

## What About Teams That Use GitHub?

Most teams aren't going to move their repos on day one. That's fine. Rift has a mirror mode.

You point Rift at your existing GitHub repo. Rift clones it and syncs every 60 seconds. Your team clones from Rift instead of GitHub. They write code in jj, push stacks to Rift, review in Rift's UI. When a stack is merged, Rift pushes the result back to GitHub.

GitHub stays the source of truth for CI and deployments. Rift is the review layer on top. Your team gets stacked review without moving anything. If it doesn't work out, you just stop using Rift and everything is still on GitHub.

This is the adoption path. Try the review experience first. Move your repos later (or don't).

## What About Git Users on the Team?

Not everyone needs to switch to jj immediately. Git users can:

Clone and fetch from Rift-hosted repos using standard Git commands. `git clone` just works.

Review in the web UI. The review experience doesn't require jj. Any team member can read diffs, leave comments, and approve changes through the browser.

Pull merged changes. When stacks are merged to main, `git pull` picks them up like any other commit.

The one thing Git users can't do in the first version is submit changes for review through Git. That requires jj (or, for mirror-mode repos, pushing directly to GitHub). Native Git push-for-review is on the roadmap.

## The Key Concepts Explained

If you're going to talk about Rift with anyone — teammates, investors, users — these are the four ideas that matter.

### Stacks, Not Pull Requests

A stack is an ordered chain of changes submitted for review. Think of it as a PR that can hold multiple independent, reviewable units of work. Each change in the stack has its own diff, its own comments, and its own approval state.

The order matters. Change B depends on Change A. Change C depends on Change B. Rift shows them in this order and lets reviewers walk through the logic step by step.

### Iterations, Not Force-Pushes

Every time you update a stack, Rift creates a new iteration. Iteration 1 is the original. Iteration 2 is after you addressed review feedback. Iteration 3 is after the next round. Nothing is overwritten. Every version is preserved.

This is fundamentally different from GitHub, where a force-push replaces the old commits and the review history is gone. On Rift, the reviewer can always look back at any iteration and see what the code looked like at that point.

### Interdiff: The Killer Feature

Interdiff shows what changed between two iterations of the same change. Not the full diff — the diff of the diff. If a reviewer approved Change A in iteration 1 and the author didn't touch it in iteration 2 (just restacked it), the interdiff for A is empty. The reviewer knows instantly that A is unchanged and doesn't need re-review.

This is what makes stacked review scale. Without it, every update to any change in the stack forces reviewers to re-read everything. With it, they only look at what actually changed.

### Mirror Mode: Try Before You Migrate

Mirror mode is the answer to "we're not moving our repo." Point Rift at your GitHub repo. Rift syncs it. Your team reviews in Rift. Merged commits go back to GitHub. If you stop using Rift, nothing is lost. It's a zero-risk trial of the review experience.

## What the MVP Ships

The first version includes:

**Stacked review.** Push a chain of changes, review each one independently, amend and restack, see iteration history, approve per-change, merge.

**Mirror mode.** Sync an existing GitHub repo into Rift. Review in Rift. Merge back to GitHub.

**Interdiff.** Compare any two iterations of a stack to see exactly what the author changed.

**Git compatibility.** Clone and fetch from any Rift repo using standard Git.

**Web UI.** Stack overview, per-change diffs, full-stack diffs, inline comments, revision graph, file browser.

The first version does not include:

- CI integration (you keep using your existing CI)
- Conflict resolution UI (resolve conflicts locally in jj)
- Git push-for-review (Git users review in the browser, submit changes via jj or GitHub)
- Webhooks or event streaming
- Full-text search

These are all on the roadmap. They're not in the first version because the first version needs to prove one thing: that stacked review with iterations and interdiff is a better way to review code.

## Glossary

Quick reference for terms used in Rift and the spec.

| Term           | Meaning                                                                                                                                                                                           |
| -------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Revision       | An immutable snapshot of code at a point in time. Same concept as a Git commit. Once created, it never changes.                                                                                   |
| Change         | A mutable reference to a revision. When you amend a revision, the change keeps the same ID (change_id) even though the underlying revision is new. This is the key concept that Git doesn't have. |
| change_id      | The stable identifier for a change. Survives amends, rebases, and restacks. It's how Rift knows "this is the same logical change, just updated."                                                  |
| Stack          | An ordered chain of changes submitted for review. Replaces the pull request. Has a status: open, merged, or closed.                                                                               |
| Iteration      | A version of a stack. Every time the author pushes an update, the iteration counter goes up. Reviewers can compare any two iterations.                                                            |
| Interdiff      | A diff between two iterations of the same change. Shows what the author actually modified since the last review, filtering out noise from restacking.                                             |
| Bookmark       | A named pointer, like main or develop. Used for integration targets. Equivalent to a Git branch, but doesn't drive the review workflow.                                                           |
| Restack        | When you amend a change, jj automatically rebases all dependent changes on top of the new version. This is automatic in jj; in Git you'd have to rebase manually.                                 |
| Mirror mode    | Running Rift as a review layer on top of an existing GitHub/GitLab repo. Rift syncs the repo, your team reviews in Rift, merged commits go back to the origin.                                    |
| delta_hash     | A fingerprint of what a change actually did (lines added/removed), independent of its position in the stack. Used to detect whether a change is truly "unchanged" across iterations.              |
| blocked_reason | Why a stack can't be merged right now. Usually "rebase required" (the base moved and there's a conflict) or "origin diverged" (someone force-pushed to the GitHub repo).                          |

## Where to Go from Here

If you're an engineer who's going to build Rift, read the MVP Specification (v1.6.1). It has the full architecture, data model, API surface, error codes, and implementation contracts. Everything in this document is explained in precise detail there.

If you're evaluating Rift as a user, the thing to focus on is the review experience: stacked changes, iterations, and interdiff. That's the product. Mirror mode is how you try it without risk.

If you're new to jj, the official site is [jj-vcs.github.io](https://jj-vcs.github.io). You don't need to master jj to understand Rift, but spending an hour with it will make the concepts click. The two ideas that matter most for Rift are stable change IDs and automatic restacking.
