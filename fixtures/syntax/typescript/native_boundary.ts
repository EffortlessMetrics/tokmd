import { dlopen, FFIType } from "bun:ffi";
import type { RuntimeOptions } from "./runtime";

type NativePointer = { run(): void };

const nativeBinding = dlopen("libtokmd.dylib", {
  run: {
    args: [FFIType.ptr],
    returns: FFIType.void,
  },
});

export function bindNative(ptr: unknown) {
  const typed = ptr as NativePointer;
  return nativeBinding.symbols.run!(typed);
}

export const loadPlugin = async (name: string) => import(`./plugins/${name}`);

export class RuntimeBridge {
  invoke(value?: unknown) {
    return (value as NativePointer).run();
  }
}

export function main(options: RuntimeOptions) {
  if (process.argv.includes("--serve")) {
    Bun.serve({ fetch: () => new Response(options.name) });
  }
}
