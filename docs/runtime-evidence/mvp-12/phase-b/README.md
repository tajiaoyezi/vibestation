# MVP-12 Phase B Runtime Evidence

Phase B delivers the Canvas rendering layer for commit rail graph geometry and
paint behavior.

## Screenshot Waiver

All screenshot and visual baseline artifact generation was skipped at the
user's explicit request during session 26. This includes the 10 PNG baseline
matrix originally listed for Phase B.

## Replacement Evidence

- `geometry.test.ts` covers measured row center alignment, merge endpoints,
  fork detection, HEAD node precedence, and ref tip geometry.
- `canvas-paint.test.ts` covers DPR clamp/backing store scale, color-token
  reads, bezier edge drawing, distinct local/remote/tag tip paint operations,
  and selected-row overlay paint.
- Manual screenshot review remains deferred to the reviewer per the user's
  waiver.
