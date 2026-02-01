# typed: false
# frozen_string_literal: true

class Tokmd < Formula
  desc "Code intelligence for humans, machines, and LLMs: receipts, metrics, and insights"
  homepage "https://github.com/EffortlessMetrics/tokmd"
  url "https://github.com/EffortlessMetrics/tokmd/archive/refs/tags/v1.3.1.tar.gz"
  sha256 "UPDATE_SHA256_AFTER_RELEASE"
  license any_of: ["MIT", "Apache-2.0"]
  head "https://github.com/EffortlessMetrics/tokmd.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "crates/tokmd")
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/tokmd --version")

    # Test that it can scan a simple directory
    (testpath/"test.rs").write('fn main() { println!("hello"); }')
    output = shell_output("#{bin}/tokmd --format json #{testpath}")
    assert_match "Rust", output
  end
end
