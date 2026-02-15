#!/bin/bash
# rust-review.sh
echo "🦀 Reviewing Rust code against Microsoft guidelines..."
echo "Guidelines location: ./rust-guidelines.txt"
echo ""
# Run standard Rust checks
cargo fmt --check
cargo clippy -- -D warnings
# Add custom checks based on guidelines
echo "✅ Standard checks complete. Review against guidelines in ./rust-guidelines.txt"