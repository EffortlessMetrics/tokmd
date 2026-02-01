"""Basic tests for tokmd Python bindings."""

import json
import pytest


def test_import():
    """Test that the module can be imported."""
    import tokmd

    assert tokmd is not None


def test_version():
    """Test version returns a valid string."""
    import tokmd

    v = tokmd.version()
    assert isinstance(v, str)
    assert len(v) > 0
    assert "." in v  # Should look like semver


def test_schema_version():
    """Test schema_version returns an integer."""
    import tokmd

    sv = tokmd.schema_version()
    assert isinstance(sv, int)
    assert sv >= 1


def test_module_attributes():
    """Test module has expected attributes."""
    import tokmd

    assert hasattr(tokmd, "__version__")
    assert hasattr(tokmd, "SCHEMA_VERSION")
    assert hasattr(tokmd, "TokmdError")


def test_run_json_version():
    """Test run_json with version mode."""
    import tokmd

    result = tokmd.run_json("version", "{}")
    data = json.loads(result)

    assert "version" in data
    assert "schema_version" in data


def test_run_json_invalid_mode():
    """Test run_json returns error for invalid mode."""
    import tokmd

    result = tokmd.run_json("invalid_mode", "{}")
    data = json.loads(result)

    assert data.get("error") is True
    assert "code" in data
    assert "message" in data


def test_run_json_invalid_json():
    """Test run_json returns error for invalid JSON."""
    import tokmd

    result = tokmd.run_json("lang", "not valid json")
    data = json.loads(result)

    assert data.get("error") is True
    assert data.get("code") == "invalid_json"


def test_lang_basic():
    """Test lang function with defaults."""
    import tokmd

    result = tokmd.lang(paths=["src"])

    assert result["mode"] == "lang"
    assert "schema_version" in result
    assert "rows" in result
    assert isinstance(result["rows"], list)


def test_lang_with_top():
    """Test lang function with top parameter."""
    import tokmd

    result = tokmd.lang(paths=["src"], top=2)

    # Should have at most 2 real rows (+ possibly "Other")
    assert len(result["rows"]) <= 3


def test_module_basic():
    """Test module function with defaults."""
    import tokmd

    result = tokmd.module(paths=["src"])

    assert result["mode"] == "module"
    assert "rows" in result
    assert isinstance(result["rows"], list)


def test_export_basic():
    """Test export function with defaults."""
    import tokmd

    result = tokmd.export(paths=["src"])

    assert result["mode"] == "export"
    assert "rows" in result
    assert isinstance(result["rows"], list)


def test_run_function():
    """Test the generic run function."""
    import tokmd

    result = tokmd.run("lang", {"paths": ["src"]})

    assert result["mode"] == "lang"
    assert "rows" in result


def test_run_function_error():
    """Test run function raises TokmdError on error."""
    import tokmd

    with pytest.raises(tokmd.TokmdError):
        tokmd.run("invalid_mode", {})


def test_diff_requires_paths():
    """Test diff requires from and to paths."""
    import tokmd

    # Should work with valid paths
    result = tokmd.diff("src", "src")
    assert result["mode"] == "diff"


class TestLangOptions:
    """Test various lang function options."""

    def test_files_option(self):
        import tokmd

        result = tokmd.lang(paths=["src"], files=True)
        assert result["args"]["with_files"] is True

    def test_children_collapse(self):
        import tokmd

        result = tokmd.lang(paths=["src"], children="collapse")
        assert result["args"]["children"] == "collapse"

    def test_children_separate(self):
        import tokmd

        result = tokmd.lang(paths=["src"], children="separate")
        assert result["args"]["children"] == "separate"


class TestModuleOptions:
    """Test various module function options."""

    def test_module_roots(self):
        import tokmd

        result = tokmd.module(paths=["src"], module_roots=["crates"])
        assert "crates" in result["args"]["module_roots"]

    def test_module_depth(self):
        import tokmd

        result = tokmd.module(paths=["src"], module_depth=3)
        assert result["args"]["module_depth"] == 3


class TestExportOptions:
    """Test various export function options."""

    def test_min_code(self):
        import tokmd

        result = tokmd.export(paths=["src"], min_code=100)
        assert result["args"]["min_code"] == 100

    def test_max_rows(self):
        import tokmd

        result = tokmd.export(paths=["src"], max_rows=10)
        assert result["args"]["max_rows"] == 10
