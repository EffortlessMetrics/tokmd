import unittest
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from release_consumer_smoke_aggregate import REQUIRED_SURFACES, aggregate_entries


def receipts(statuses):
    return [{"kind": surface, "status": status} for surface, status in statuses.items()]


def all_passed():
    return {surface: "passed" for surface in REQUIRED_SURFACES}


class AggregateContractTests(unittest.TestCase):
    def test_all_required_surfaces_pass(self):
        result = aggregate_entries(receipts(all_passed()), {}, "stable")
        self.assertEqual(result["overall"], "passed")

    def test_explicit_failure_blocks(self):
        statuses = all_passed()
        statuses["wasm"] = "failed"
        result = aggregate_entries(receipts(statuses), {}, "stable")
        self.assertEqual(result["overall"], "failed")

    def test_absent_receipt_blocks(self):
        statuses = all_passed()
        del statuses["nix"]
        result = aggregate_entries(receipts(statuses), {}, "stable")
        self.assertEqual(result["entries"]["nix"]["status"], "failed")
        self.assertEqual(result["overall"], "failed")

    def test_unavailable_required_surface_blocks(self):
        statuses = all_passed()
        statuses["nix"] = "unavailable"
        result = aggregate_entries(receipts(statuses), {}, "stable")
        self.assertEqual(result["overall"], "failed")

    def test_rc_policy_can_explicitly_exclude_not_supported_surface(self):
        statuses = all_passed()
        statuses["nix"] = "not_supported"
        result = aggregate_entries(receipts(statuses), {}, "rc", {"nix"})
        self.assertEqual(result["overall"], "passed")

    def test_stable_cannot_exclude_not_supported_surface(self):
        statuses = all_passed()
        statuses["nix"] = "not_supported"
        result = aggregate_entries(receipts(statuses), {}, "stable", {"nix"})
        self.assertEqual(result["overall"], "failed")


if __name__ == "__main__":
    unittest.main()
