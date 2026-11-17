# 73. Drop selling plans from Storefront offer details

Status: Accepted

Supersedes the selling-plan rows of ADR 56 and the selling-plan page sizes of
ADR 55. Other page sizes are unchanged.

## Decision

The Anker, Minisforum and Ugreen OfferDetails and ProductVariants queries no
longer select `product.sellingPlanGroups` or `variant.sellingPlanAllocations`.
The ProductSellingPlanGroups, VariantSellingPlanAllocations and SellingPlans
continuation operations are removed. Raw and extracted models drop the
selling-plan types. `requiresSellingPlan` is kept because it is a plain product
field that needs no extra access scope.

## Reason and rollout

ADR 72 acquires tokens for every storefront again. Acquired tokens often lack the
`unauthenticated_read_selling_plans` scope, and Shopify then rejects the whole
detail request with ACCESS_DENIED, as ADR 70 already observed for Ugreen EU.
Offers do not use subscription data, so the field is not worth the failures.

Raw artifacts captured before this change still deserialize. Their
`sellingPlanGroups` and `sellingPlanAllocations` fields are accepted and dropped
rather than rejected as unknown, so stored artifacts can be extracted again.
Extracted output no longer contains selling-plan fields. No queue rows or stored
artifacts are rewritten. Deploy updated workers before retrying failed commands.
