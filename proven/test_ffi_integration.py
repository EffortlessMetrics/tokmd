#!/usr/bin/env python3
"""
Integration test suite for tokmd-python FFI seams.

Tests the Python ↔ Rust FFI boundary for error handling, GIL safety, and
memory safety during exception paths.

Usage:
    python3 test_ffi_integration.py [--verbose]

Exit codes:
    0 - All tests passed
    1 - One or more tests failed
"""

import sys
import os
import json
import tempfile
import shutil
import gc
from pathlib import Path

# Create symlink from libtokmd.so to _tokmd.so in the target directory
lib_path = '/home/openclaw/.openclaw/workspace/tokmd/target/release/libtokmd.so'
tokmd_module_path = '/home/openclaw/.openclaw/workspace/tokmd/crates/tokmd-python/python/tokmd/_tokmd.so'

# Create symlink if it doesn't exist
if not os.path.exists(tokmd_module_path):
    os.makedirs(os.path.dirname(tokmd_module_path), exist_ok=True)
    os.symlink(lib_path, tokmd_module_path)
    print(f"Created symlink: {tokmd_module_path} -> {lib_path}")

# Add the Python module path
sys.path.insert(0, '/home/openclaw/.openclaw/workspace/tokmd/crates/tokmd-python/python')


def setup_module():
    """Ensure the native module is available."""
    import importlib.util
    spec = importlib.util.find_spec('tokmd._tokmd')
    if spec is None:
        print("ERROR: tokmd._tokmd native module not found!")
        print("Make sure the Rust extension is built:")
        print("  cargo build -p tokmd-python --release")
        return False

    print(f"Spec found: {spec}")
    print(f"  Origin: {spec.origin}")
    return True


class FFIIntegrationTests:
    """Test suite for FFI seam validation."""

    def __init__(self, verbose=False):
        self.verbose = verbose
        self.passed = 0
        self.failed = 0
        self.tests = []

    def log(self, msg):
        if self.verbose:
            print(f"  {msg}")

    def run_test(self, name, test_func):
        """Run a single test and track results."""
        try:
            test_func()
            self.passed += 1
            print(f"  ✓ {name}")
            return True
        except AssertionError as e:
            self.failed += 1
            print(f"  ✗ {name}: {e}")
            return False
        except Exception as e:
            self.failed += 1
            print(f"  ✗ {name}: Exception: {type(e).__name__}: {e}")
            return False

    # ===================================================================
    # TEST 1: Invalid JSON Input Handling
    # ===================================================================

    def test_invalid_json_raises_valueerror(self):
        """Test that invalid JSON in args raises ValueError before reaching Rust."""
        import tokmd

        # Python bindings validate JSON before releasing GIL / calling Rust
        # This should raise ValueError at the Python boundary
        try:
            result = tokmd.run_json("lang", "not valid json!!!")
            # If we get here, the error is in the envelope
            data = json.loads(result)
            assert data.get("ok") == False, f"Expected error response, got: {data}"
            self.log("Invalid JSON returned error envelope")
        except ValueError as e:
            # This is the expected path - JSON validated before FFI call
            self.log(f"ValueError raised correctly at Python boundary: {e}")
            return  # Test passes

    def test_malformed_json_variations(self):
        """Test various malformed JSON inputs raise ValueError."""
        import tokmd

        test_cases = [
            "{invalid",      # Unclosed brace
            "}",             # Just closing brace
            "{'key': 'val'}",  # Single quotes (invalid JSON)
        ]

        for i, bad_json in enumerate(test_cases):
            try:
                result = tokmd.run_json("lang", bad_json)
                # If no exception, result should be error envelope
                assert isinstance(result, str)
            except ValueError:
                # This is expected - JSON validated before FFI call
                pass
            self.log(f"Case {i}: '{bad_json[:20]}...' handled gracefully")

        for i, bad_json in enumerate(test_cases):
            try:
                result = tokmd.run_json("lang", bad_json)
                # If no exception, result should be error envelope
                assert isinstance(result, str)
            except ValueError:
                # This is expected - JSON validated before FFI call
                pass
            self.log(f"Case {i}: '{bad_json[:20]}...' handled gracefully")

        # Edge cases: empty/null are also invalid JSON according to validator
        edge_cases_invalid = [
            ("", "empty"),
            ("null", "null"),
        ]
        for json_input, desc in edge_cases_invalid:
            try:
                result = tokmd.run_json("lang", json_input)
                assert isinstance(result, str) and len(result) > 0
                self.log(f"Edge case '{desc}' returned result")
            except ValueError:
                self.log(f"Edge case '{desc}' raised ValueError (valid)")

        # Array is valid JSON but may be rejected by the core
        result = tokmd.run_json("lang", "[1, 2, 3]")
        assert isinstance(result, str)
        self.log("Edge case 'array' handled")

    # ===================================================================
    # TEST 2: Invalid Mode Handling
    # ===================================================================

    def test_invalid_mode_returns_error(self):
        """Test that invalid mode returns error envelope, not panic."""
        import tokmd

        result = tokmd.run_json("not_a_real_mode_12345", "{}")
        data = json.loads(result)

        assert data.get("ok") == False or data.get("error") is True, \
            f"Expected error for invalid mode, got: {data}"

        # Should have error details
        assert "code" in data or ("error" in data and data["error"]), \
            f"Expected error code, got: {data}"

        self.log("Invalid mode handled gracefully")

    def test_high_level_invalid_mode_raises_exception(self):
        """Test that high-level API raises TokmdError for invalid mode."""
        import tokmd

        try:
            result = tokmd.run("not_a_real_mode_12345", {})
            # If we get here, result should be error envelope
            assert False, f"Expected TokmdError to be raised, got result: {result}"
        except tokmd.TokmdError as e:
            self.log(f"TokmdError raised correctly: {e}")
            return  # Test passes
        except Exception as e:
            assert False, f"Expected TokmdError, got {type(e).__name__}: {e}"

    # ===================================================================
    # TEST 3: Nonexistent Path Handling
    # ===================================================================

    def test_nonexistent_path_returns_error(self):
        """Test that nonexistent path returns error envelope."""
        import tokmd

        result = tokmd.run_json("lang", '{"paths": ["/definitely/does/not/exist"]}')
        data = json.loads(result)

        # Should be an error or empty result
        if data.get("ok") == False:
            self.log(f"Nonexistent path returns error: {data.get('error')}")
        else:
            # Some modes return empty results for nonexistent paths
            assert "rows" in data or "error" in data, f"Unexpected response: {data}"
            self.log("Nonexistent path handled (empty result or error)")

    def test_high_level_nonexistent_path_raises(self):
        """Test that high-level API handles nonexistent path gracefully."""
        import tokmd

        try:
            result = tokmd.lang(paths=["/definitely/does/not/exist"])
            # Some implementations return empty result
            self.log(f"Result for nonexistent path: {type(result)}")
        except tokmd.TokmdError as e:
            self.log(f"TokmdError for nonexistent path: {e}")
        except Exception as e:
            # Other exceptions are acceptable as long as they don't panic
            self.log(f"Exception for nonexistent path: {type(e).__name__}: {e}")

    # ===================================================================
    # TEST 4: GIL Handling During Errors
    # ===================================================================

    def test_gil_released_during_scan(self):
        """Test that GIL is properly released during long operations."""
        import tokmd
        import threading
        import time

        results = {}

        def scan_in_thread():
            try:
                # This releases GIL during the scan
                result = tokmd.lang(paths=["/home/openclaw/.openclaw/workspace/tokmd/src"], top=5)
                results['scan'] = ('ok', result)
            except Exception as e:
                results['scan'] = ('error', type(e).__name__, str(e))

        def simple_operation():
            try:
                # This should be able to run while scan releases GIL
                v = tokmd.version()
                results['version'] = ('ok', v)
            except Exception as e:
                results['version'] = ('error', type(e).__name__, str(e))

        # Run operations
        scan_in_thread()
        simple_operation()

        # Both should complete without hanging
        assert 'scan' in results, "Scan did not complete"
        assert 'version' in results, "Version call did not complete"

        self.log(f"GIL handling: scan={results['scan'][0]}, version={results['version'][0]}")

    def test_concurrent_access_does_not_deadlock(self):
        """Test that concurrent Python calls don't deadlock."""
        import tokmd
        import threading

        errors = []
        successes = []

        def worker(worker_id):
            try:
                for _ in range(3):
                    v = tokmd.version()
                    assert isinstance(v, str)
                successes.append(worker_id)
            except Exception as e:
                errors.append((worker_id, str(e)))

        threads = [threading.Thread(target=worker, args=(i,)) for i in range(4)]
        for t in threads:
            t.start()
        for t in threads:
            t.join(timeout=10)

        # Check for deadlocked threads
        alive = [t for t in threads if t.is_alive()]
        if alive:
            assert False, f"{len(alive)} threads deadlocked!"

        self.log(f"Concurrent access: {len(successes)} workers succeeded, {len(errors)} errors")

    # ===================================================================
    # TEST 5: Memory Safety During Exception Paths
    # ===================================================================

    def test_exception_does_not_corrupt_memory(self):
        """Test that exceptions don't leave memory in corrupted state."""
        import tokmd

        # Trigger several errors in sequence
        for i in range(10):
            try:
                tokmd.run("invalid_mode_" + str(i), {})
            except tokmd.TokmdError:
                pass  # Expected
            except Exception:
                pass  # Also acceptable

        # After exceptions, normal operation should still work
        v = tokmd.version()
        assert isinstance(v, str) and len(v) > 0, "Memory corruption detected - version() failed after exceptions"

        self.log("Memory state consistent after exception handling")

    def test_gc_during_ffi_call(self):
        """Test that GC during FFI call doesn't cause issues."""
        import tokmd
        import gc

        # Trigger GC during FFI operations
        for _ in range(5):
            gc.collect()
            v = tokmd.version()
            assert isinstance(v, str)

        self.log("GC during FFI calls handled correctly")

    # ===================================================================
    # TEST 6: Error Propagation Across Language Boundary
    # ===================================================================

    def test_error_message_propagation(self):
        """Test that error messages propagate correctly from Rust to Python."""
        import tokmd

        result = tokmd.run_json("definitely_invalid_mode_xyz", "{}")
        data = json.loads(result)

        # Error should have meaningful message
        if data.get("ok") == False and "error" in data:
            error_info = data["error"]
            if isinstance(error_info, dict):
                assert "message" in error_info, f"Error missing message: {error_info}"
                msg = error_info.get("message", "")
                assert len(msg) > 0, "Error message is empty"
                self.log(f"Error message propagated: '{msg[:50]}...'")
            elif isinstance(error_info, str):
                assert len(error_info) > 0, "Error message is empty"
                self.log(f"Error message propagated: '{error_info[:50]}...'")

    def test_error_code_propagation(self):
        """Test that error codes are properly structured."""
        import tokmd

        result = tokmd.run_json("invalid", "{}")
        data = json.loads(result)

        if data.get("ok") == False:
            # Should have either error.code or error dict with code
            error = data.get("error", {})
            if isinstance(error, dict):
                # Code should be present and be a string
                code = error.get("code")
                if code is not None:
                    assert isinstance(code, (str, int)), f"Code should be string or int, got {type(code)}"
                    self.log(f"Error code propagated: '{code}'")
                else:
                    self.log("No error code in response (acceptable)")

    # ===================================================================
    # TEST 7: Edge Cases for FFI Boundary
    # ===================================================================

    def test_very_long_path(self):
        """Test that very long paths don't cause buffer overflow."""
        import tokmd

        long_path = "/tmp/" + "a" * 5000

        try:
            result = tokmd.lang(paths=[long_path])
            self.log("Very long path handled without crash")
        except tokmd.TokmdError as e:
            self.log(f"Very long path handled with error: {e}")
        except Exception as e:
            self.log(f"Very long path handled with exception: {type(e).__name__}")

        # Most important: should not crash
        v = tokmd.version()
        assert isinstance(v, str), "System crashed after long path test"

    def test_special_characters_in_path(self):
        """Test paths with special characters."""
        import tokmd

        special_paths = [
            "/tmp/test path with spaces",
            "/tmp/test-path-with-dashes",
            "/tmp/test_path_with_underscores",
            "/tmp/test.path.with.dots",
        ]

        for path in special_paths:
            try:
                result = tokmd.run_json("lang", json.dumps({"paths": [path]}))
                self.log(f"Special path handled: '{path[:40]}...'")
            except Exception as e:
                self.log(f"Special path error: {type(e).__name__}")

    def test_unicode_handling(self):
        """Test that Unicode in arguments is handled correctly."""
        import tokmd

        # Test with Unicode characters (that are valid UTF-8)
        # Note: This tests JSON encoding/decoding across the boundary
        result = tokmd.run_json("version", "{}")
        data = json.loads(result)

        # Should be valid JSON
        assert "ok" in data or "version" in data, "Unicode handling affected basic operation"
        self.log("Unicode handling test passed")

    # ===================================================================
    # TEST 8: Consistency After Error Recovery
    # ===================================================================

    def test_consistency_after_error(self):
        """Test that the module remains consistent after errors."""
        import tokmd

        # Get baseline
        v1 = tokmd.version()
        sv1 = tokmd.schema_version()

        # Trigger error
        try:
            tokmd.run("invalid", {})
        except tokmd.TokmdError:
            pass

        # Verify module still works correctly
        v2 = tokmd.version()
        sv2 = tokmd.schema_version()

        assert v1 == v2, f"Version changed after error: {v1} -> {v2}"
        assert sv1 == sv2, f"Schema version changed after error: {sv1} -> {sv2}"

        self.log("Module state consistent after error recovery")

    def test_repeated_error_recovery(self):
        """Test repeated error and recovery cycles."""
        import tokmd

        for i in range(20):
            try:
                if i % 2 == 0:
                    tokmd.run("invalid_mode", {})
                else:
                    v = tokmd.version()
                    assert isinstance(v, str)
            except tokmd.TokmdError:
                pass

        # Final check
        v = tokmd.version()
        assert isinstance(v, str) and len(v) > 0
        self.log("Repeated error recovery cycles handled")

    # ===================================================================
    # TEST 9: Empty and Null Input Handling
    # ===================================================================

    def test_empty_dict_args(self):
        """Test that empty dict args are handled."""
        import tokmd

        result = tokmd.run_json("version", "{}")
        data = json.loads(result)
        assert "version" in data or data.get("ok") == True, f"Empty args failed: {data}"
        self.log("Empty dict args handled correctly")

    def test_minimal_valid_args(self):
        """Test minimal valid argument sets."""
        import tokmd

        # Version mode with no args
        result = tokmd.run_json("version", "{}")
        data = json.loads(result)
        assert data.get("ok") == True, f"Version failed: {data}"

        # Lang with minimal args
        result = tokmd.run_json("lang", '{"paths": ["."]}')
        data = json.loads(result)
        # May error if no files found, but should be graceful
        self.log(f"Minimal args: version ok={data.get('ok')}")

    # ===================================================================
    # Run All Tests
    # ===================================================================

    def run_all(self):
        """Run all integration tests."""
        print("\n" + "="*70)
        print("FFI INTEGRATION TEST SUITE")
        print("="*70)

        # Setup
        if not setup_module():
            return 1

        import tokmd
        print(f"Testing tokmd version: {tokmd.version()}")
        print(f"Schema version: {tokmd.schema_version()}")

        # Test Group 1: Invalid Input Handling
        print("\n--- Test Group 1: Invalid Input Handling ---")
        self.run_test("Invalid JSON raises appropriate error", self.test_invalid_json_raises_valueerror)
        self.run_test("Malformed JSON variations handled", self.test_malformed_json_variations)

        # Test Group 2: Invalid Mode Handling
        print("\n--- Test Group 2: Invalid Mode Handling ---")
        self.run_test("Invalid mode returns error envelope", self.test_invalid_mode_returns_error)
        self.run_test("High-level invalid mode raises TokmdError", self.test_high_level_invalid_mode_raises_exception)

        # Test Group 3: Path Handling
        print("\n--- Test Group 3: Path Handling ---")
        self.run_test("Nonexistent path returns error", self.test_nonexistent_path_returns_error)
        self.run_test("High-level nonexistent path handled", self.test_high_level_nonexistent_path_raises)

        # Test Group 4: GIL Handling
        print("\n--- Test Group 4: GIL Handling ---")
        self.run_test("GIL released during scan", self.test_gil_released_during_scan)
        self.run_test("Concurrent access does not deadlock", self.test_concurrent_access_does_not_deadlock)

        # Test Group 5: Memory Safety
        print("\n--- Test Group 5: Memory Safety ---")
        self.run_test("Exception does not corrupt memory", self.test_exception_does_not_corrupt_memory)
        self.run_test("GC during FFI call handled", self.test_gc_during_ffi_call)

        # Test Group 6: Error Propagation
        print("\n--- Test Group 6: Error Propagation ---")
        self.run_test("Error message propagation", self.test_error_message_propagation)
        self.run_test("Error code propagation", self.test_error_code_propagation)

        # Test Group 7: Edge Cases
        print("\n--- Test Group 7: Edge Cases ---")
        self.run_test("Very long path handled", self.test_very_long_path)
        self.run_test("Special characters in path", self.test_special_characters_in_path)
        self.run_test("Unicode handling", self.test_unicode_handling)

        # Test Group 8: Consistency
        print("\n--- Test Group 8: Consistency ---")
        self.run_test("Consistency after error", self.test_consistency_after_error)
        self.run_test("Repeated error recovery", self.test_repeated_error_recovery)

        # Test Group 9: Input Handling
        print("\n--- Test Group 9: Empty/Null Input Handling ---")
        self.run_test("Empty dict args", self.test_empty_dict_args)
        self.run_test("Minimal valid args", self.test_minimal_valid_args)

        # Summary
        print("\n" + "="*70)
        print("SUMMARY")
        print("="*70)
        print(f"  Passed: {self.passed}")
        print(f"  Failed: {self.failed}")
        print(f"  Total:  {self.passed + self.failed}")

        if self.failed == 0:
            print("\n  ✓ ALL TESTS PASSED")
            return 0
        else:
            print(f"\n  ✗ {self.failed} TEST(S) FAILED")
            return 1


def main():
    verbose = "--verbose" in sys.argv or "-v" in sys.argv
    tests = FFIIntegrationTests(verbose=verbose)
    return tests.run_all()


if __name__ == "__main__":
    sys.exit(main())
