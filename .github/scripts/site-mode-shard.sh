#!/usr/bin/env bash
# The browser suite with DWELLER_PROCESS=site (Q2.14a, `.github/workflows/site-mode.yml`).
#
# run SHARD SHARDS: runs the shard's tests once, with no re-run of a failed
#   test, and writes the names that passed and failed to target/site-mode/ok.txt
#   and failed.txt. A failing test is the measurement, not an error: the exit
#   is non-zero only when the shard did not run to its `test result:` line, or
#   when the names read from the log do not add up to that line.
# summary SHARDS DIR: adds the ok.txt and failed.txt of every DIR/*/ and
#   prints the Markdown summary: passed over total, then the failing names
#   grouped by the name's prefix up to its second underscore.
set -eo pipefail

out=target/site-mode

shard_names() {
  cargo test --release --test browser -- --list --format terse | sed -n 's/: test$//p' \
    | awk 'NR == FNR { t[$1] = $2; next } { k = $1; if (!(k in t)) sub(/.*::/, "", k); print (k in t ? t[k] : 10), k, $1 }' tests/browser-times.txt - \
    | LC_ALL=C sort -k1,1nr -k2,2 \
    | awk -v n="$2" -v s="$1" '{ b = 0; for (i = 1; i < n; i++) if (load[i] < load[b]) b = i; load[b] += $1 } b == s { print $3 }'
}

# The libtest lines `test NAME ... ok|FAILED` between `running N tests` and
# the failure report, so that output a failed test printed is never counted.
split_log() {
  awk -v ok="$2/ok.txt" -v failed="$2/failed.txt" '
    /^running [0-9]+ tests?$/ { on = 1; next }
    /^failures:$/ || /^test result:/ { on = 0 }
    on && NF == 4 && $1 == "test" && $3 == "..." && $4 == "ok" { print $2 > ok }
    on && NF == 4 && $1 == "test" && $3 == "..." && $4 == "FAILED" { print $2 > failed }
  ' "$1"
}

count() {
  echo $(($(wc -l < "$1")))
}

run() {
  names="$(shard_names "$1" "$2")"
  if [ -z "$names" ]; then
    echo "::error::shard $(($1 + 1))/$2 has no tests"
    exit 1
  fi
  mkdir -p "$out"
  : > "$out/ok.txt"
  : > "$out/failed.txt"
  status=0
  AYE="$(command -v aye)" cargo test --release --test browser -- --exact $names 2>&1 | tee "$out/log" || status=$?
  split_log "$out/log" "$out"
  result="$(grep '^test result:' "$out/log" | tail -n 1 || true)"
  if [ -z "$result" ]; then
    echo "::error::shard $(($1 + 1))/$2 did not finish (cargo exited $status)"
    exit 1
  fi
  passed="$(echo "$result" | sed -n 's/.* \([0-9][0-9]*\) passed;.*/\1/p')"
  failed="$(echo "$result" | sed -n 's/.* \([0-9][0-9]*\) failed;.*/\1/p')"
  if [ "$passed" != "$(count "$out/ok.txt")" ] || [ "$failed" != "$(count "$out/failed.txt")" ]; then
    echo "::error::the log lists $(count "$out/ok.txt") ok and $(count "$out/failed.txt") FAILED, but says: $result"
    exit 1
  fi
}

# Every DIR/*/NAME, one after another; nothing when no shard finished.
shard_files() {
  find "$dir" -mindepth 2 -maxdepth 2 -name "$1" -exec cat {} +
}

summary() {
  shards="$1"
  dir="$2"
  mkdir -p "$dir"
  finished="$(find "$dir" -mindepth 1 -maxdepth 1 -type d | wc -l)"
  passed="$(shard_files ok.txt | wc -l)"
  failed="$(shard_files failed.txt | wc -l)"
  echo "## \`DWELLER_PROCESS=site\`: $((passed)) / $((passed + failed)) passed"
  echo
  echo "$((failed)) failing; $((finished)) of $shards shards finished."
  shard_files failed.txt \
    | awk '{ n = $1; sub(/.*::/, "", n); k = split(n, p, "_"); print (k > 2 ? p[1] "_" p[2] : n) "\t" $1 }' \
    | LC_ALL=C sort \
    | awk -F '\t' '
        $1 != g { if (g != "") flush(); g = $1; c = 0; body = "" }
        { c++; body = body "- `" $2 "`\n" }
        END { if (g != "") flush() }
        function flush() { printf "\n### %s (%d)\n\n%s", g, c, body }'
}

"$@"
