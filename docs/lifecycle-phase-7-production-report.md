# Phase 7 production coverage expansion result

**Direct conditional assurance expanded.** Three distinct real source token accounts executed locally; two swapped successfully at a second verified DLMM pool and all three transferred to a distinct real observed recipient. The large source’s four swaps failed with verified rollback. **Most newly measured raw amount is Transfer evidence, not market exit evidence. OfficialTransition remains NotTested.** No mainnet transaction was submitted.

## Exact before/after coverage

Population remains 17,957 token accounts, 10,155 positive public balances and 17,950 distinct observed owner authorities. These are accounts and authorities, not identified people.

| Quantity | Phase 6 before | Phase 7 after | Change |
| --- | ---: | ---: | ---: |
| Proven entities | 1 | 4 | +3 |
| PartiallyProven entities | 0 | 0 | 0 |
| Untested entities | 12,378 | 12,375 | -3 |
| Unsupported entities | 5,578 | 5,578 | 0 |
| Measured entities / observed authorities | 1 / 1 | 4 / 4 | +3 / +3 |
| Represented public amount raw | 8,741,534,482,051 | 8,741,534,482,051 | 0 |
| Any-path measured maximum raw | 60,354 | 1,457,718,144,803 | +1,457,718,084,449 |
| Represented raw without any-path evidence | 8,741,534,421,697 | 7,283,816,337,248 | -1,457,718,084,449 |
| SecondaryMarketExit measured raw | 60,354 | 387,113 | +326,759 |
| Transfer measured raw | 0 | 1,457,718,084,449 | +1,457,718,084,449 |
| Directly measured secondary-market venues | 1 | 2 | +1 |
| Measured path types | 1 | 2 | +1 |
| Measured coarse account classes | 1 | 1 | 0 |

The three new entities retain the Phase 6 narrow `Proven` meaning: a successful full exact-input local execution at matching source public balance on at least one requested path. The large entity is proven through Transfer only; its SecondaryMarketExit path remains Untested. This does not establish signing access, inclusion, market sale, liquidity, redemption, official conversion, legal entitlement or every path. Zero balances remain explicit and never vacuously proven.

Each entity contributes its maximum measured exact amount across paths and venues. The small and medium amounts appear in both path totals but enter the global total once. Outputs cannot be added as simultaneous proceeds or pool capacity. A maximum is a numeric summary of exact tested points, not an assurance that every smaller amount succeeds.

Percentages are descriptive fractions with explicit denominators, rounded to 12 decimals: proven accounts 1/17,957 (0.005568858941%) → 4/17,957 (0.022275435763%); proven positive accounts 1/10,155 (0.009847365830%) → 4/10,155 (0.039389463319%). Any-path measured raw / 8,741,534,482,051 is 0.000000690428% → 16.675769543621%. Secondary-market measured raw / that same denominator is only 0.000004428433% after expansion. None is a safety score or a fraction of assured liquidity.

## Architecture, score and immutable selection

Observed population → validated Phase 6 coverage → class/path gaps → eligibility-tagged candidates → immutable selected plan → current finalized captures → actual fresh-VM evidence → verified coverage delta. Selection cannot consume execution results. Public update checks content hashes and fresh-replays all results before granting coverage.

The selector version is `lifecycle-gap-v1`; its serializable configuration, weights and amount strategy are saved with the plan. The score is `100*new_entity + floor(20*min(marginal_raw,1,000,000,000)/1,000,000,000) + 40*new_class + 60*new_market_venue + 80*new_path + 30*new_entity_path_context + 25*new_shape + 5*feasible - 1,000*redundancy`. Binary components and marginal raw are computed against virtual predicted per-entity maxima. Weights express development priorities, not monetary value or probability.

The production budget is six groups, three distinct sources, at most two groups per source and two path types. It first covers deterministic positive-wallet rank quartiles 0, 1 and 3. Within declared constraints, ties sort by decreasing score, decreasing marginal raw, then entity, context and path in increasing order. Amount capping and bucket constraints prevent the largest source from occupying every slot.

There are **30,465 candidate groups**: 1 ExecutableCandidate, 21,352 CaptureRequired, 9,111 Unsupported and 1 Invalid. Unsupported/invalid candidates have no score. Fifty unsupported venue contexts are recorded separately rather than multiplied across all entities. Zero public balances are excluded from positive execution selection. The old fully evidenced entity/venue context remains executable but redundant and is not selected.

The six selected groups below were frozen before capture. Labels S/M/L are abbreviations for the exact entities listed next, not inferred economic classes. All are CaptureRequired. Expected gains remain unchanged even when realized gains differ.

| Order | Source / quartile | Path / context | Score components (entity, amount, class, venue, path, tuple, shape, feasibility, penalty) | Total | Expected new global raw | Realized new global raw |
| ---: | --- | --- | --- | ---: | ---: | ---: |
| 0 | S / 0 | Transfer / recipient | 100, 0, 0, 0, 80, 30, 25, 5, 0 | 240 | 17621 | 17621 |
| 1 | M / 1 | SecondaryMarketExit / second pool | 100, 0, 0, 60, 0, 30, 0, 5, 0 | 195 | 309138 | 309138 |
| 2 | L / 3 | SecondaryMarketExit / second pool | 100, 20, 0, 0, 0, 30, 25, 5, 0 | 180 | 1457717757690 | 0 |
| 3 | S / 0 | SecondaryMarketExit / second pool | 0, 0, 0, 0, 0, 30, 0, 5, 0 | 35 | 0 | 0 |
| 4 | M / 1 | Transfer / recipient | 0, 0, 0, 0, 0, 30, 0, 5, 0 | 35 | 0 | 0 |
| 5 | L / 3 | Transfer / recipient | 0, 0, 0, 0, 0, 30, 0, 5, 0 | 35 | 0 | 1457717757690 |

Group 0 gains a new source, path and balance shape. Group 1 gains a new source and exact venue; its bucket matches an already measured shape. Group 2 gains a large-bucket source/shape with capped amount priority, but realizes zero because swaps fail. Groups 3–5 gain distinct entity/path/context observations with no predicted global raw duplication. Group 5 realizes the large source’s amount through Transfer after its previously selected swap fails; its expected gain remains zero. No successful fallback was selected retrospectively.

All three new sources belong to the same coarse `WalletCompatible / Initialized / nondelegated / nonconfidential / unverified original role` class. Balance quartiles yield three deterministic state-shape fingerprints. **Other authority classes were not executed and no class peers inherit proof.**

## Exact entities, venues and authority assumptions

| Label | Real source token account | Original retained owner | Planned and captured public balance raw |
| --- | --- | --- | ---: |
| S | `741ZXYKzgjPZucRhPUQFK7kKAgP1ESJwimhumgGpuPHs` | `2wCvQzHiDHAHTvzwPeof9H3uEzq8Bzvg38DFbvZMGkuj` | 17621 |
| M | `E6NHqVMSHrssPiKGgKnfxNqjoTnQXMoL5xDUvDBXiFdE` | `8uN1R2nEei4kzst55bRqGhJEA7kNYxmJyeWpvCzZcpcD` | 309138 |
| L | `ENc8TdLutJ2ziFnz9x4uV8pEmdnk6iWYaBHaTAPCejpV` | `6GJbPKBtovsrMEEMcic5KMi5tswh9qSyT5ZYLMqEwNgt` | 1457717757690 |

Every original authority is on curve, System-owned, empty and non-executable in its captured final batch. Every case records **original owner locally assumed to sign**; `signer_possession_known=false`, `signer_assumed_locally=true`. This proves neither key possession nor authorization and identifies no human holder. Delegates and permanent delegate are preserved but unused.

Second pool actually executed: `22PthLk8TYnurtbWKRyECFd99cHHbfsbPNHfeMetzfZg`, deployed Meteora DLMM `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo`, SPACEX Token-2022 → legacy-token USDC. Its target vault is `8H7LiWHtwA9S6BcQGjoAYhYo8aVVmdsuVeznSZdKGoAB`; paired vault `J1o8ikH5wKTnNyAjeYjKRfwCEBTVLbiV5v15pL7QZHpb`.

The original pool `v4D5b4knJ83WzErtDiugxChUZUfWxe2FtDLguarvgFc` retains its independently measured Phase 6 evidence, freshly validated by the planning CLI. The six new groups use the second pool or Transfer context; there is no new current capture of the first pool and no invented cross-venue execution for a selected source. The cumulative two-venue count means exact independent evidence at both pools, not a Meteora brand guarantee.

Transfer context: distinct real initialized SPACEX recipient `124XHuTYUNnCf2ABCNcCEdQFpHQ7Y6MnPe9NeouDB1az`, owner `D1ZN9Wj1fRSUQfCjhvnu1hqDMT7hzjzBBpi12nVniYD6`. Its captured prebalance is 60,354 raw in each independent transfer bank. No recipient or token amount was fabricated. A transfer to that account is not a successor-token transition.

## Production discovery and exact captures

Read-only public RPC origin: `https://api.mainnet-beta.solana.com`. Finalized mint-position scans at slots 448015698 and 448015699 returned 48 and 4 concrete DLMM pools. The inventory retains both complete responses and one additional venue’s independently verified account/PDA/mint/vault proof. The bounded verifier selects the lexicographically first additional compatible pool before probe selection. Unverified or unsupported pools retain zero execution assurance. Discovery status was SecondVenueVerifiedExecutionPending; after direct successful swaps the delta records **SecondVenueExecuted**.

Each selected group captured its own coherent finalized route batch. Captures completed on 2026-09-18; all six succeeded. Transfer batches contain seven accounts; market batches contain 22 entries, including one genuine null optional bitmap PDA. Full raw records, provider, parameters, owners, bytes, hashes, executable code and Clock remain in fixtures.

| Group | Capture completed UTC | Final slot | Clock epoch | Final batch entries | Fixture bytes | SHA-256 |
| ---: | --- | ---: | ---: | ---: | ---: | --- |
| 0 | 2026-09-18T07:39:15.727432Z | 448018365 | 1037 | 7 | 1850592 | `d99ec5e1c0c6aedf4d1a675ddaff9c5cbf7dab11b77c478ef045416c6185c264` |
| 1 | 2026-09-18T07:39:30.719680Z | 448018422 | 1037 | 22 | 5488101 | `21870077ff6082cc6c40c1d036b7057d8176ec1b45fcfe1a2387eb7012ae2487` |
| 2 | 2026-09-18T07:39:46.094715Z | 448018480 | 1037 | 22 | 5488103 | `51457a1baaf3bfcb8c9e178de88376f60eb53dca9387106af4b5a7f476a18f39` |
| 3 | 2026-09-18T07:40:01.485006Z | 448018537 | 1037 | 22 | 5488100 | `bc0ee60ac9e0326eabbbf72dba4afd00ff4bed218bcbb02609318720bc11cc50` |
| 4 | 2026-09-18T07:40:03.392352Z | 448018544 | 1037 | 7 | 1850593 | `a83e9a55c1e6fc2cd2d0e363e32ad9f7ec6cb8434ca269f32ce24618be70ef68` |
| 5 | 2026-09-18T07:40:04.763095Z | 448018550 | 1037 | 7 | 1850595 | `122fd13fb1ffda01d8117d1224497986a676d661e3d06aa5fd6e4e6105dcd727` |

Clock slots range 448018365–448018550, epoch 1037; exact per-case unix timestamps are retained in VM evidence. Policy interpretation remains 2026-09-17T20:00:00Z. These are current finalized captures, not the Phase 4 historical bank or old Phase 5 epoch-1036 state. All three source public balances match their population amounts, enabling the unchanged conditional amount join. Matching quantities do not make the new execution bank historical.

Deployed ProgramData and allocated ELF input hashes match these exact captures: DLMM deployment slot 423977638, SHA-256 `d296c6771cec945601027613ca637c1be6721044c5859009d41c453477844c1f`; Token-2022 slot 427147035, `0999dbf708971e723b08d1caafc988826a59c6001ed6dc02260da07defbe1469`; legacy token slot 419472000, `8190d3f7ceb6cb7a7a8d8924bff89f9f611e15ce1f806f2b6237f3311a98f697`. Canonical loaders and code are retained, but no source-to-bytecode equivalence is asserted.

## Actual amount matrix and failures

Every point resets its captured bank. Exact integer points are 1%, 25%, 50% and full planned public amount; each balance-plus-one control is excluded from positive coverage. Swap minimum output is one raw USDC, a test threshold rather than a production slippage recommendation. Transfer output is SPACEX public credit; swap output is USDC credit. Withheld fees are separate, never counted twice.

| Group / path | Exact input raw | Status | Actual public output raw | Withheld raw | DLMM total / protocol raw | CU |
| --- | ---: | --- | ---: | ---: | --- | ---: |
| 0 / Transfer | 176 | Succeeded | 175 | 1 | — | 4622 |
| 0 / Transfer | 4405 | Succeeded | 4382 | 23 | — | 4622 |
| 0 / Transfer | 8810 | Succeeded | 8765 | 45 | — | 4622 |
| 0 / Transfer | 17621 | Succeeded | 17532 | 89 | — | 4622 |
| 0 / Transfer | 17622 | Indeterminate | — | — | — | — |
| 1 / SecondaryMarketExit | 3091 | Succeeded | 1853 | 16 | 62 / 6 | 39436 |
| 1 / SecondaryMarketExit | 77284 | Succeeded | 46349 | 387 | 1538 / 153 | 39436 |
| 1 / SecondaryMarketExit | 154569 | Succeeded | 92699 | 773 | 3076 / 307 | 39436 |
| 1 / SecondaryMarketExit | 309138 | Succeeded | 185399 | 1546 | 6152 / 615 | 39436 |
| 1 / SecondaryMarketExit | 309139 | Indeterminate | — | — | — | — |
| 2 / SecondaryMarketExit | 14577177576 | Failed | 0 | 0 | — | 407661 |
| 2 / SecondaryMarketExit | 364429439422 | Failed | 0 | 0 | — | 407661 |
| 2 / SecondaryMarketExit | 728858878845 | Failed | 0 | 0 | — | 410756 |
| 2 / SecondaryMarketExit | 1457717757690 | Failed | 0 | 0 | — | 411311 |
| 2 / SecondaryMarketExit | 1457717757691 | Indeterminate | — | — | — | — |
| 3 / SecondaryMarketExit | 176 | Succeeded | 105 | 1 | 4 / 0 | 39437 |
| 3 / SecondaryMarketExit | 4405 | Succeeded | 2641 | 23 | 88 / 8 | 39436 |
| 3 / SecondaryMarketExit | 8810 | Succeeded | 5282 | 45 | 176 / 17 | 39439 |
| 3 / SecondaryMarketExit | 17621 | Succeeded | 10567 | 89 | 351 / 35 | 39436 |
| 3 / SecondaryMarketExit | 17622 | Indeterminate | — | — | — | — |
| 4 / Transfer | 3091 | Succeeded | 3075 | 16 | — | 4622 |
| 4 / Transfer | 77284 | Succeeded | 76897 | 387 | — | 4622 |
| 4 / Transfer | 154569 | Succeeded | 153796 | 773 | — | 4622 |
| 4 / Transfer | 309138 | Succeeded | 307592 | 1546 | — | 4622 |
| 4 / Transfer | 309139 | Indeterminate | — | — | — | — |
| 5 / Transfer | 14577177576 | Succeeded | 14504291688 | 72885888 | — | 4622 |
| 5 / Transfer | 364429439422 | Succeeded | 362607292224 | 1822147198 | — | 4622 |
| 5 / Transfer | 728858878845 | Succeeded | 725214584450 | 3644294395 | — | 4622 |
| 5 / Transfer | 1457717757690 | Succeeded | 1450429168901 | 7288588789 | — | 4622 |
| 5 / Transfer | 1457717757691 | Indeterminate | — | — | — | — |

Actual cases: **20 Succeeded, 4 Failed, 6 Indeterminate**. Twenty-four complete transactions executed; six controls failed insufficient-balance preconditions and executed no transaction. Successful full swaps yield 10,567 raw USDC for S and 185,399 for M. Full transfers yield 17,532 / 307,592 / 1,450,429,168,901 raw SPACEX; associated withheld fees are 89 / 1,546 / 7,288,588,789 raw. Each full successful source ends at zero locally; mainnet funds were not moved.

All four L swaps (14,577,177,576; 364,429,439,422; 728,858,878,845; 1,457,717,757,690 raw) return `InstructionError(1, Custom(6036))`, with deployed logs identifying `BitmapExtensionAccountIsNotProvided`. Captured optional bitmap PDA `GV1952Q4mrY89f55B6csBecoS6CwLLtnaHqaLMpbb9GW` is genuinely absent. The bounded route cannot service those selected amounts; this is not a proof of unavailable global liquidity or that another complete route cannot sell. All watched token, withheld, pool, oracle and bin states roll back byte-for-byte; measured debits/credits are zero. These cases gain no assurance and were not replaced or recaptured after failure.

Six Indeterminate controls are S 17,622, M 309,139 and L 1,457,717,757,691 raw, once per selected path. Their insufficient-source-balance precondition blockers remain explicit. There are no omitted Failed or Indeterminate cases. All positive successful token/fee/event/bin reconciliations are retained. No quote substitutes for execution.

Transfer executes actual deployed Token-2022 TransferChecked, applies captured epoch-1037 transfer fees independently checked by the SPL formula, and verifies debit = recipient public credit + recipient withheld fee. Mint bytes and source withheld fee remain unchanged. Paused/hooked/frozen states are rejected and covered by controlled tests. Confidential account and required incoming memo semantics remain outside the supported adapter slice and are rejected. Secondary-market successes reconcile vault public/withheld, bin reserves, event payloads and protocol accumulators; host fee is zero. Fees are not presumed identical to the old pool.

LiteSVM 0.16 uses the existing generic backend and pinned mainnet/default feature/sysvar profile. Native System/compute-budget/loaders, transaction instruction sysvar and a synthetic local fee payer are runtime inputs. Each transaction pays 10,000 synthetic lamports; payer funds and assumed signatures are local only. No private keys, authorization, inclusion, future liquidity or full validator-bank equivalence are established.

## Remaining high-value gaps

| Gap | Represented raw without path evidence | Meaning / next capability |
| --- | ---: | --- |
| OfficialTransition | 8,741,534,482,051 | No independently verified executable successor transition; NotTested |
| Redemption | 8,741,534,482,051 | No supported verified redemption adapter |
| Withdrawal | 8,741,534,482,051 | Requires actual vault/LP/authority-specific instructions and account proof |
| SecondaryMarketExit | 8,741,534,094,938 | More exact sources, route depth and venues remain unmeasured |
| Transfer | 7,283,816,397,602 | Compatible peers remain untested; non-wallet authority support absent |
| ProgramOwnedAuthority, all currently unsupported execution models | 299,279,967,199 | 165 entities / 68 positive; program-mediated authority paths needed |
| Unknown authority model | 367,785,541,678 | 5,413 entities / 2,969 positive; authority capability evidence needed |

These overlap by path and class and must never be added as separate capital. Verified protocol vault observations overlap population accounts and add no holders or reserves to the denominator. Program ownership alone does not establish LP rights or an executable withdrawal. All untested class peers, unverified venues, issuer entitlement/KYC and real signer capabilities remain gaps. The planner preserves top balance entity IDs per class/path as development guidance, without scoring unsupported capabilities as executable.

## Artifacts, exact files and evidence hashes

The versioned delta retains the Phase 6 report by digest, explicit capability overrides and changed entity rows. It avoids copying every old entity and execution payload. Baseline + WalletCompatible Transfer capability override + changed rows reconstructs the updated portfolio view; no positive proof follows from the override alone. Original Phase 2–6 files and semantics remain unchanged.

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| [probes/spacex-phase7-before.json](../probes/spacex-phase7-before.json) | 894 | `6948886fa6e11d5aa11f8ee36761a4466643cf8e31eb9d49faff5c6fc2504472` |
| [probes/spacex-phase7-selector-config.json](../probes/spacex-phase7-selector-config.json) | 518 | `8e28c100df4ff663d02105c182ee86093006a910b16f23bc4ed69ea873c619b5` |
| [probes/spacex-phase7-venues-verified.json](../probes/spacex-phase7-venues-verified.json) | 120891 | `3ab6a481cb2c87b241e3f4671c8cb1ac772dac5bcac39d035f26d90a58879c0d` |
| [probes/spacex-lifecycle-expansion-plan.json](../probes/spacex-lifecycle-expansion-plan.json) | 33678509 | `6758fbd4e3d99c44e726c638fe2bcbe514ce702957af6dfd121bc8d23cd4aade` |
| [probes/phase7-captures/capture-manifest.json](../probes/phase7-captures/capture-manifest.json) | 2417 | `5357ef0ff0436649e27c626f81f16be99022a48ce232a640d9114228d0befdc6` |
| [reports/phase7-evidence/execution-index.json](../reports/phase7-evidence/execution-index.json) | 15847 | `e14c449199c972b91b5390b325b8ed8285f07efa70c5a41df960e2a142ee3c52` |
| [reports/spacex-lifecycle-coverage-phase7.json](../reports/spacex-lifecycle-coverage-phase7.json) | 57874 | `79b78e877b63c9fa3fd5ef80755b0759c47ae8ac9771a56cc3046a387217e07f` |
| [reports/spacex-phase7-mutation-results.json](../reports/spacex-phase7-mutation-results.json) | 3919 | `0d9266b56c3a322f483e43499b15dd15783c7dc3cedfe692a905f771ea284488` |
| [reports/spacex-phase7-performance.json](../reports/spacex-phase7-performance.json) | 1222 | `a56431cd53d77fa1d6a53bea20eb38f3ce7211f6e2db8bcb2ac5ef2812a4a137` |

Phase 6 baseline SHA-256: `f3ff991165d23f9ff851b8e7d9c141a92aac4668bf26e2bc83926a12a2e4d920`; population SHA-256: `6802afb871035a0883a04196c7c9542d021d97514f43748b8f63747ea3ce016e`. The before record was saved before execution; its plan digest remains identical after capture, execution and update. Adjacent checksums and [complete Phase 7 artifact checksum list](../reports/spacex-phase7-artifacts.sha256) verify from repository root.

New source/guide files are `engine/src/expansion/mod.rs`, `selector.rs`, `discovery.rs`, `pipeline.rs`; `engine/src/probe/token_transfer.rs`; `engine/tests/coverage_expansion.rs`; `scripts/test-phase7-mutations.py`; `docs/lifecycle-phase-7-expansion.md`; and this production report. Existing phase files updated are `AGENTS.md`, `README.md`, `engine/src/lib.rs`, `engine/src/main.rs`, `engine/src/probe/mod.rs`, `capture.rs`, `meteora_dlmm.rs`. Generic executor behavior and all previous tests remain unchanged. No dependency was added.

Exact new data files are the nine artifact paths above, `reports/spacex-phase7-validation.json`, `reports/spacex-phase7-worktree-status.txt`, six fixtures `probes/phase7-captures/fixtures/group-0.json` through `group-5.json`, ten mutation logs `reports/phase7-mutations/01.txt` through `10.txt`, and their artifact checksum list. Adjacent `.sha256` files accompany the before/config/inventory/plan/manifest/index/delta/mutation/performance/validation JSON. The following table names every new content-addressed execution file exactly; none is duplicated inside the delta.

| Exact case | Status | Evidence file (name is SHA-256) |
| --- | --- | --- |
| group-0-raw-176 | Succeeded | [ab06340581cda069bc7875e974a7d51dde84ceba4d7ef5f2051960ccb4290cb9](../reports/phase7-evidence/results/ab06340581cda069bc7875e974a7d51dde84ceba4d7ef5f2051960ccb4290cb9.json) |
| group-0-raw-4405 | Succeeded | [8911247a21051627a2e47e0a20f26b46829f4300f0380db3f85690f39d3a4e54](../reports/phase7-evidence/results/8911247a21051627a2e47e0a20f26b46829f4300f0380db3f85690f39d3a4e54.json) |
| group-0-raw-8810 | Succeeded | [c1bec2bdaa04a8ce75c7daeb4a8970826f7115c25d6cf33cb7838df2c2a870d2](../reports/phase7-evidence/results/c1bec2bdaa04a8ce75c7daeb4a8970826f7115c25d6cf33cb7838df2c2a870d2.json) |
| group-0-raw-17621 | Succeeded | [85e1ee3067ce9a33bba222b2d88559d6a9bde91091b0ba10d661c37ac182895a](../reports/phase7-evidence/results/85e1ee3067ce9a33bba222b2d88559d6a9bde91091b0ba10d661c37ac182895a.json) |
| group-0-raw-17622 | Indeterminate | [493edfdecddb83193831108b12fbc3dc6595105a7ac37a7f7e5a0978605f4463](../reports/phase7-evidence/results/493edfdecddb83193831108b12fbc3dc6595105a7ac37a7f7e5a0978605f4463.json) |
| group-1-raw-3091 | Succeeded | [6a39c3d3abc7fd6c38455d99a75b56ea1cc9e827fef190a03c9e8bbaa9fb4cc2](../reports/phase7-evidence/results/6a39c3d3abc7fd6c38455d99a75b56ea1cc9e827fef190a03c9e8bbaa9fb4cc2.json) |
| group-1-raw-77284 | Succeeded | [a136906766be4d5f8921cfe53f0e05a3aad47410a2f08610c906ddca944679ad](../reports/phase7-evidence/results/a136906766be4d5f8921cfe53f0e05a3aad47410a2f08610c906ddca944679ad.json) |
| group-1-raw-154569 | Succeeded | [e618824624be8baf28c1fd9dc694fe44d5d5d5316fee52f2f2961b4da0b22a84](../reports/phase7-evidence/results/e618824624be8baf28c1fd9dc694fe44d5d5d5316fee52f2f2961b4da0b22a84.json) |
| group-1-raw-309138 | Succeeded | [674eb93145aa130e7a88aa81b2570709d2817962ea3b767d1ef457ebafdbe984](../reports/phase7-evidence/results/674eb93145aa130e7a88aa81b2570709d2817962ea3b767d1ef457ebafdbe984.json) |
| group-1-raw-309139 | Indeterminate | [d499f81e0e23c122acaf45fadac6d8be3838dad1ad05ad5925ec9d30784e6a6f](../reports/phase7-evidence/results/d499f81e0e23c122acaf45fadac6d8be3838dad1ad05ad5925ec9d30784e6a6f.json) |
| group-2-raw-14577177576 | Failed | [6105d7609ea6840cec226188199159b02c9089a83ebadc83abe7b2ce82d53530](../reports/phase7-evidence/results/6105d7609ea6840cec226188199159b02c9089a83ebadc83abe7b2ce82d53530.json) |
| group-2-raw-364429439422 | Failed | [87be5f200f4491d658cdc03abfc29937103abc73d0f068ecd152467a3de179dc](../reports/phase7-evidence/results/87be5f200f4491d658cdc03abfc29937103abc73d0f068ecd152467a3de179dc.json) |
| group-2-raw-728858878845 | Failed | [97aa887081db308ed5538403efbe76640a039100d1d56c08eac12cabc1cb31f2](../reports/phase7-evidence/results/97aa887081db308ed5538403efbe76640a039100d1d56c08eac12cabc1cb31f2.json) |
| group-2-raw-1457717757690 | Failed | [46a947ef45c55f478f1b01416c63a8c5c43a87c4f90e120863499612679fbfef](../reports/phase7-evidence/results/46a947ef45c55f478f1b01416c63a8c5c43a87c4f90e120863499612679fbfef.json) |
| group-2-raw-1457717757691 | Indeterminate | [b020661764ec104c48b338ecaec6006915153a1490b83e04bf6de2e512911d6f](../reports/phase7-evidence/results/b020661764ec104c48b338ecaec6006915153a1490b83e04bf6de2e512911d6f.json) |
| group-3-raw-176 | Succeeded | [429614ecbf4d015213b77e2047efe97e41bb5c7ded4e5518b71791d611eab8f0](../reports/phase7-evidence/results/429614ecbf4d015213b77e2047efe97e41bb5c7ded4e5518b71791d611eab8f0.json) |
| group-3-raw-4405 | Succeeded | [19cd07e2310a0707f48a7c0aa70648796066c29dffcfb750cc02f1dc574f0f0f](../reports/phase7-evidence/results/19cd07e2310a0707f48a7c0aa70648796066c29dffcfb750cc02f1dc574f0f0f.json) |
| group-3-raw-8810 | Succeeded | [b3105e98c3c84aa5e4ef8879cfcab09a5b42dd4ba469a9b251bf42732605cb6d](../reports/phase7-evidence/results/b3105e98c3c84aa5e4ef8879cfcab09a5b42dd4ba469a9b251bf42732605cb6d.json) |
| group-3-raw-17621 | Succeeded | [697e63336a6468cb340cbbfdc614c5e27c2e1f64b80ccde7c1b7e65a17c822a6](../reports/phase7-evidence/results/697e63336a6468cb340cbbfdc614c5e27c2e1f64b80ccde7c1b7e65a17c822a6.json) |
| group-3-raw-17622 | Indeterminate | [7f80ff9d726b076e3abaff78a4a360c88d750fd75200d548cf7a004dc6b54bd7](../reports/phase7-evidence/results/7f80ff9d726b076e3abaff78a4a360c88d750fd75200d548cf7a004dc6b54bd7.json) |
| group-4-raw-3091 | Succeeded | [6056a14b7829e815500a5a70667965964324b4b2511d8c4ea45af49e22091f28](../reports/phase7-evidence/results/6056a14b7829e815500a5a70667965964324b4b2511d8c4ea45af49e22091f28.json) |
| group-4-raw-77284 | Succeeded | [1b3ac53efcc86222514244c5e62e2269cc4f190ec06c7311896063c91cf7ff7b](../reports/phase7-evidence/results/1b3ac53efcc86222514244c5e62e2269cc4f190ec06c7311896063c91cf7ff7b.json) |
| group-4-raw-154569 | Succeeded | [616c3a1914d7dd1d0f1aea43ce62fb024dab296422f38e78dd847301d510bedd](../reports/phase7-evidence/results/616c3a1914d7dd1d0f1aea43ce62fb024dab296422f38e78dd847301d510bedd.json) |
| group-4-raw-309138 | Succeeded | [4401be5f6ace5a897a97d3b6c19bc11718e2dd8fa52d4cf3fcd894acd7d8b72c](../reports/phase7-evidence/results/4401be5f6ace5a897a97d3b6c19bc11718e2dd8fa52d4cf3fcd894acd7d8b72c.json) |
| group-4-raw-309139 | Indeterminate | [b95d56dea931d767f0097532f1ad633ee1ed9ce5df6f0720cdbe8cded2c4c449](../reports/phase7-evidence/results/b95d56dea931d767f0097532f1ad633ee1ed9ce5df6f0720cdbe8cded2c4c449.json) |
| group-5-raw-14577177576 | Succeeded | [eefd93e2806f1bcebc625d380579f5f7d242069da482d92ca8db80ce1acf21ac](../reports/phase7-evidence/results/eefd93e2806f1bcebc625d380579f5f7d242069da482d92ca8db80ce1acf21ac.json) |
| group-5-raw-364429439422 | Succeeded | [70e901479f612116cfa7ab88e38f845f72a8d659b6d8c4a06e65a11a14a69ddc](../reports/phase7-evidence/results/70e901479f612116cfa7ab88e38f845f72a8d659b6d8c4a06e65a11a14a69ddc.json) |
| group-5-raw-728858878845 | Succeeded | [ada3fde1d361ad2475f9d320f1541bbd86c245908c2eac1bc4180cae8903a02a](../reports/phase7-evidence/results/ada3fde1d361ad2475f9d320f1541bbd86c245908c2eac1bc4180cae8903a02a.json) |
| group-5-raw-1457717757690 | Succeeded | [41d81f4d13f8cf3ba5799213dbb5aa2c8d99fd31e550a4a0a3069a8e0f039299](../reports/phase7-evidence/results/41d81f4d13f8cf3ba5799213dbb5aa2c8d99fd31e550a4a0a3069a8e0f039299.json) |
| group-5-raw-1457717757691 | Indeterminate | [b5ba6afc5b012909771396761dd7093afe8e524a2d0c98f8e4f1d6059afbebf4](../reports/phase7-evidence/results/b5ba6afc5b012909771396761dd7093afe8e524a2d0c98f8e4f1d6059afbebf4.json) |

## Validation and mutation results

`make test`: **205 passed, 0 failed, 0 ignored**, including both actual synthetic SBF builds. `make fmt-check`, `make lint` and `git diff --check`: passed. Suite breakdown: fixture v1 7, fixture v2 7, engine 71, Phase 7 6, Phase 5 12, Phase 4 12, Phase 6 16, Phase 3 11, Phase 2 13, scenario 2, upgrade integration 42, interface 6. No execution tests were skipped or artifacts hidden. [Validation record](../reports/spacex-phase7-validation.json) retains exact counts and replay checks.

All 30 selected evidence files and the execution index replay byte-identically offline with `SOLANA_RPC_URL`, `RPC_URL`, `ARCHIVE_RPC_URL` and `SOLANA_ARCHIVE_RPC_URL` unset. Fresh coverage update is also byte-identical, as are saved canonical JSON and stdout. Original snapshot/report/probe checksums verify. All five new CLI commands reject existing output paths before performing work and leave those artifacts unchanged. Candidate selection is byte-identical after reversed population/coverage entity ordering.

Ten faults were actually injected into valid Rust code, one at a time. Every named test failed by assertion, not compiler error; source bytes were restored exactly. Full portable logs and their digests are bound in the mutation report.

| Injected fault | Specific test that caught it |
| --- | --- |
| Predict independent contexts by summing amounts | `predicted_independent_contexts_use_maxima` |
| Propagate representative evidence to class peers | `a_class_sample_never_proves_its_peers` |
| Inherit all same-brand venue evidence | `second_venue_has_no_brand_or_first_venue_inheritance` |
| Score unsupported candidates | `unsupported_and_invalid_cannot_be_scored` |
| Rewrite expected gains from measured results | `execution_never_rewrites_expected_selection_gains` |
| Conflate entity and authority counts | `distinct_accounts_with_one_authority_stay_distinct` |
| Allow failed probes to gain assurance | `failed_indeterminate_unsupported_and_controls_gain_nothing` |
| Sum independent executions as capacity | `independent_swaps_are_maxima_never_summed_capacity` |
| Label current capture as a historical bank | `current_capture_cannot_be_labeled_a_historical_bank` |
| Inherit swap and transfer into official transition | `official_transition_and_redemption_cannot_inherit_swap_or_transfer` |

New tests cover bounded amount arithmetic/overflow/zero, eligibility, decomposed scoring, shape/class/venue gains, maximum/non-extrapolation, authority/entity separation, immutable expectations, production ordering determinism and real multi-entity evidence. Token-transfer tests execute deployed bytecode and cover real success/fee/replay, insufficient input, wrong mint/authority/freeze, paused/hooked mint, distinct real destination and genuine program failure rollback. Public update tests reject changed fixture bytes and fabricated economic deltas even after result digests are recomputed. No prior assertion was weakened.

## Performance and storage

Whole-command wall time and peak child-process RSS were measured; Darwin reports RSS in bytes. These include canonical parsing/hashing, validation and fresh execution; some commands overlap other validation work, so they are observations rather than isolated benchmark guarantees.

| Stage | Wall seconds | Peak RSS MiB |
| --- | ---: | ---: |
| plan | 211.543 | 691.19 |
| capture | 88.912 | 855.45 |
| execute | 125.852 | 649.88 |
| update | 135.678 | 590.17 |
| offline-execute | 147.756 | 745.56 |
| offline-update | 149.923 | 701.56 |

Candidate generation indexes entities, authorities and shapes; sorting is O(N log N), followed by a bounded group-budget scan. It avoids an all-population pairwise comparison and avoids multiplying unsupported venue contexts. Canonical hashing and full baseline fresh validation dominate portions of CLI runtime; fixture parsing and plan hashing still incur repeated work. The measured population-scale workflow takes minutes and hundreds of MiB, not a subsecond claim.

Route fixtures, manifest, execution index and all 30 result files total 23,128,522 raw JSON bytes. The plan separately records 30,465 candidates, costing about 32 MB; the updated coverage delta is only 57,874 bytes instead of another 61 MB population report. Each selected group retains its own coherent fixture, including repeated deployed bytes; no speculative code/bank deduplication alters fixture identity. Evidence bodies are content-addressed and stored once, with digest references in the delta.

## Worktree status and justified claims

Branch `main`, HEAD `3787a697bbdd9ed08d87e5b63800724d4de2d55c`. No commit, push or broadcast occurred. Earlier phases remain uncommitted in the shared tree; Phase 7 adds to that tree. [Exact worktree status](../reports/spacex-phase7-worktree-status.txt) records tracked modifications and untracked paths at completion.

The acceptance evidence now supports deterministic pre-execution gap prioritization, bounded six-group capture, direct execution of three real sources, successful second-venue execution, a real Transfer adapter, exact non-double-counted before/after coverage and immutable expected-versus-realized reporting. These are measured conditional local execution facts under the retained original-owner signer assumption.

It still prohibits blanket exitability, continuous amount intervals, simultaneous pooled liquidity/proceeds, peer/class/brand inheritance, actual mainnet funds movement, signature authorization, future liquidity, full validator equivalence, issuer conversion/redemption, legal entitlement/KYC and vault/LP withdrawal claims. No USD value, successor integration, private-key acquisition, transaction submission or UI was introduced.

See [architecture, scoring and replay CLI](lifecycle-phase-7-expansion.md) for the assurance contract and exact staged workflow.
