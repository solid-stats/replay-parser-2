---
status: complete
quick_id: 260510-hu6
implementation_commit: d0a2b33
---

# Quick Task 260510-hu6 Summary

Scaled staging parser workers from one replica to two replicas.

Changed:

- `deploy/k8s/staging/deployment.yaml` now sets `spec.replicas: 2`.
- Live staging deployment was scaled with `kubectl scale deployment/replay-parser-2 --replicas=2`.

Verification:

- `deploy/replay-parser-2` reported `2/2` Ready after rollout.
- Both parser pods reported `1/1 Running` with zero restarts.

