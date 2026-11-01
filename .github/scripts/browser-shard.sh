#!/usr/bin/env bash
set -eo pipefail

shard="$1"
shards="$2"
log=target/browser-tests.log
flaky=target/flaky-tests.txt
export RUSTC_BOOTSTRAP=1

run_tests() {
  AYE="$(command -v aye)" cargo test --release --test browser -- --exact "$@" -Z unstable-options --report-time
}

if [ "$RUNNER_OS" = macOS ]; then col=3; else col=2; fi
names="$(cargo test --release --test browser -- --list --format terse | sed -n 's/: test$//p' \
  | awk -v c=$col 'NR == FNR { t[$1] = $c; next } { print ($1 in t ? t[$1] : 10), $1 }' tests/browser-times.txt - \
  | LC_ALL=C sort -k1,1nr -k2,2 \
  | awk -v n="$shards" -v s="$shard" '{ b = 0; for (i = 1; i < n; i++) if (load[i] < load[b]) b = i; load[b] += $1 } b == s { print $2 }')"
if [ -z "$names" ]; then
  echo "::error::shard $((shard + 1))/$shards has no tests"
  exit 1
fi

mkdir -p target
if run_tests $names 2>&1 | tee "$log"; then
  exit 0
fi
failed="$(sed -n 's/^test \([^ ]*\) \.\.\. FAILED.*/\1/p' "$log")"
if [ -z "$failed" ]; then
  exit 1
fi
echo "re-running the failed tests once:" $failed
run_tests $failed
echo "$failed" > "$flaky"
for name in $failed; do
  echo "::warning title=flaky test::$name failed, then passed on re-run"
  echo "- flaky test \`$name\` failed, then passed on re-run" >> "$GITHUB_STEP_SUMMARY"
done
