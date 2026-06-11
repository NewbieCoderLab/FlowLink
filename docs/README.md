# Cross-Device Mouse MVP Documentation

This folder contains the product and technical design documents for a macOS and Windows desktop MVP similar to Synergy, Barrier, and Mouse Without Borders.

Recommended reading order:

1. [中文总方案](./00-master-plan-zh-CN.md)
2. [PRD](./01-prd.md)
3. [Technical Architecture](./02-technical-architecture.md)
4. [Module Breakdown](./03-module-breakdown.md)
5. [Data Structures](./04-data-structures.md)
6. [Network Protocol](./05-network-protocol.md)
7. [Local Storage](./06-local-storage.md)
8. [Project Structure](./07-project-structure.md)
9. [MVP Roadmap](./08-mvp-roadmap.md)
10. [Risk Assessment](./09-risk-assessment.md)
11. [V2/V3 Evolution](./10-v2-v3-evolution.md)
12. [Next Steps](./11-next-steps.md)

Recent additions:

- `11-next-steps.md`：在 V1 骨架就绪后，把 Phase 0~7 拆为 9 个 Sprint 的实施清单，并锁定 Noise XX + Ed25519 公钥 pin 作为 V1 加密通道。

Primary recommendation: build V1 with **Tauri + Rust**.

V1 product boundary:

- macOS and Windows only.
- LAN only.
- No account system.
- No public relay or NAT traversal.
- No file transfer.
- Mouse movement, click, and wheel only.
- Keyboard sharing is reserved for a later version.
