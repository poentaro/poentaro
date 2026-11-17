---
title: "The main editor must not get its text as a controlled value prop, or fast typing gets scrambled"
author: Wongjiahau
date: 2026-11-12
branch: fix/editor-fast-typing
cost: { minutes: 11, tokens: 25376 }
id: l-7d904d2b
scope: repo
status: active
src: agent
touches:
  - web/e2e/features/fast-typing-keeps-order.feature
  - web/e2e/features/steps/fast-typing-keeps-order.steps.ts
  - web/src/components/Editor.tsx
  - web/src/components/useEditorValueSync.ts
---

## Decided
The main editor receives `defaultValue`, and a layout effect writes in outside changes. A layout effect runs in the same task as the render, so a `value` that differs from the editor's text there is always an outside change, never a lagging echo of typing.

## Tried and ruled out
- Controlled `value` on @monaco-editor/react: it syncs in a passive useEffect. When a keystroke lands before that effect runs, it writes the older text back, so characters get dropped.
- Saving the caret in a layout effect and restoring it in a passive effect: the restore can run after the next keystroke, so the caret moves back one character and letters get reordered ("voice" → "oivce").

## Watch out
Any new editor-sync logic must not compare `value` against the editor's text in a passive effect. Test with `page.keyboard.type(text, { delay: 0 })`; slower typing hides the race.

In the code: web/src/components/useEditorValueSync.ts, web/src/components/Editor.tsx, web/e2e/features/fast-typing-keeps-order.feature (issue #149, PR #150).
