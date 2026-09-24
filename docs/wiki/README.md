# Wiki

Durable knowledge about the product. Written by the Lead PO and Lead Designer,
read by everyone.

| File | Written by | Contents |
|---|---|---|
| `product-brief.md` | `/create-product` | the problem, the users, the scope, the constraints |
| `stack.md` | `/plan-product` | pinned technology choices, each tied to a constraint |
| `environment.md` | `/setup-environment` | what to install on a fresh machine, and how to verify it |
| `architecture.md` | `/plan-product` | components, data model, decisions and their alternatives |
| `corpus.md` | MC-033 | the calibration corpus's marking rule, its diagonal-gutter tolerance and its tag vocabulary — the one written source of truth for `fixtures/corpus/` |
| `v2-candidates.md` | Lead PO, at v1's close | what v1 deferred, and the measurements a v2 attempt must not re-run. The index to `panel-gutter-search.md`, `chrome-row-search.md`, `panel-edge-search.md`, `gutter-band-rescore.md`, `region-row-search.md` and `reader-furniture-search.md`, which are spike records rather than durable knowledge |
| `design/` | Lead Designer | tokens, components, layout, accessibility floor, voice |
| `audits/` | Mutation Tester, or anyone auditing the harness | test-quality and harness audits. Structure from `audits/TEMPLATE.md`, which keeps what was **decided** apart from what was **measured** |

Keep these at the altitude where they stay true for months. Anything that
changes per story belongs in the story file, not here.

When requirements change, edit `product-brief.md` and re-run `/plan-product` -
it diffs against what exists rather than starting over.
