# Contributing

1. Read architecture, data-boundary, and threat-model docs.
2. Start with a linked issue and write the contract invariant before implementation.
3. Add deterministic tests for success, rejection, authorization, and edge cases.
4. Use Stellar Testnet for experiments; do not deploy Mainnet without explicit approval.
5. Document reproducible commands and deployment evidence in pull requests.

Never commit secret keys, seed phrases, tokens, or real user data. Never put personally identifiable or sensitive accommodation/payment information in contract storage or events. Do not treat an on-chain event as proof that an off-chain claim is true. Use the Hut4Devs 🛖 commit convention where practical.
