# ADR 67: Recover Anker extraction context from recorded evidence

Status: Accepted

Extends ADR 66 for historical Anker artifacts without top-level market context.
The extracted context remains required. Raw artifact schemas remain unchanged.

When top-level context is present, preserve it. Otherwise collect country and
language from recorded response extensions.context, and currency from variant
prices/comparison prices and product price/comparison ranges. Each field must
have exactly one distinct, nonempty observed value. Missing, malformed or
conflicting evidence fails extraction. Storefront defaults, request variables
and filesystem names are not substitutes for response evidence.

Recovery happens in memory during raw-to-extracted conversion. It neither
rewrites raw dumps nor supplies acquisition time. Other brands retain their
existing required-context behavior. Independent validation failures, including
missing media IDs, remain errors.
