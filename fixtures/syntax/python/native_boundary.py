import ctypes
from ctypes import c_void_p
import importlib
import subprocess
from pathlib import Path


class NativeBridge:
    def __init__(self, library: str):
        self.library = ctypes.CDLL(library)

    def load_plugin(self, name: str):
        if name:
            return importlib.import_module(name)
        raise RuntimeError("missing plugin")


def run_command(command: list[str]) -> int:
    try:
        with open("/tmp/tokmd.log", "w") as handle:
            handle.write("run")
        return subprocess.run(command, check=True).returncode
    except OSError:
        return eval("0")


def main() -> None:
    path = Path("libtokmd.so")
    bridge = NativeBridge(str(path))
    bridge.load_plugin("tokmd_plugin")


if __name__ == "__main__":
    main()
