# Friction Item: Wrong Repo Intake
- Observation: PR closed because the work was applied to the wrong repo in the topology. Normal implementations belong in EffortlessMetrics/tokmd-swarm, which is then imported into tokmd by a merge commit.
- Impact: Blocked the PR from landing. Required aborting the work.
- Proposed Action: When assigning future work or running planning tasks, the orchestrator/Jules should ensure the agent is deployed against `EffortlessMetrics/tokmd-swarm` for core library patches. Alternatively, port this change (BTreeMap to FxHashMap optimization in tokmd-analysis) to a tokmd-swarm PR.
