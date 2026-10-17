#!/bin/bash
# Runs a build or test on the user's Mac without taking it over. One
# capped job runs at a time on the machine (the others wait for it), and
# it starts only under a load limit, at nice 10 and a QoS clamp, with
# cargo and the test runners at a few jobs. Its whole process tree is
# killed above a memory limit and stopped while macOS reports thermal or
# critical memory pressure. On Linux only nice, the job counts and the
# memory limit apply. Each run ends with one line: peak memory and time.
#
#     scripts/capped.sh cargo test --release --test browser -- window_ > target/x.log 2>&1
#
# CAPPED_JOBS (4) cargo jobs; AYE_TEST_JOBS and RUST_TEST_THREADS (3)
# test workers; CAPPED_MEM_GB (12) memory of the whole tree;
# CAPPED_MAX_LOAD (8) the 1-minute load to wait under, at most 10
# minutes; CAPPED_WAIT_MIN (60) minutes to wait for the running job;
# CAPPED_QOS utility (default) or background (efficiency cores only).

set -u

[ $# -gt 0 ] || { echo "usage: capped.sh COMMAND [ARGS...]" >&2; exit 64; }

dir=$HOME/.cache/capped
mkdir -p "$dir"

if [ -z "${CAPPED_LOCKED:-}" ]; then
    if command -v lockf >/dev/null; then
        if ! lockf -s -t 0 "$dir/lock" true; then
            echo "capped: waiting for: $(cat "$dir/holder" 2>/dev/null)" >&2
        fi
        CAPPED_LOCKED=1 exec lockf -k -t $((${CAPPED_WAIT_MIN:-60} * 60)) "$dir/lock" /bin/bash "$0" "$@"
    fi
    export CAPPED_LOCKED=1
fi
echo "$$ $(date +%H:%M:%S) $PWD: $*" > "$dir/holder"

load1() {
    if [ -r /proc/loadavg ]; then cut -d' ' -f1 /proc/loadavg; else sysctl -n vm.loadavg | awk '{print $2}'; fi
}
max_load=${CAPPED_MAX_LOAD:-8}
waited=0
while [ "$waited" -lt 600 ] && awk -v l="$(load1)" -v m="$max_load" 'BEGIN { exit !(l > m) }'; do
    [ "$waited" -eq 0 ] && echo "capped: load $(load1) > $max_load, waiting (at most 10 min)" >&2
    sleep 15
    waited=$((waited + 15))
done

export CARGO_BUILD_JOBS=${CAPPED_JOBS:-4}
export AYE_TEST_JOBS=${AYE_TEST_JOBS:-3}
export RUST_TEST_THREADS=${RUST_TEST_THREADS:-3}

if [ "$(uname)" = Darwin ]; then
    taskpolicy -c "${CAPPED_QOS:-utility}" nice -n 10 "$@" &
else
    nice -n 10 "$@" &
fi
child=$!

tree() {
    ps -Ao pid=,ppid=,rss= | awk -v root="$child" '
        { par[$1] = $2; rss[$1] = $3; pid[NR] = $1 }
        END {
            if (!(root in par)) exit
            t[root] = 1
            for (more = 1; more;) {
                more = 0
                for (i in pid) { q = pid[i]; if (!(q in t) && (par[q] in t)) { t[q] = 1; more = 1 } }
            }
            s = 0; out = ""
            for (q in t) { s += rss[q]; out = out " " q }
            print s out
        }'
}

pressure() {
    [ "$(uname)" = Darwin ] || return 1
    [ "$(sysctl -n kern.memorystatus_vm_pressure_level 2>/dev/null)" = 4 ] && { echo "critical memory pressure"; return 0; }
    pmset -g therm 2>/dev/null | grep -Eq 'warning level (set to|is) [1-9]|Speed_Limit[^0-9]*[0-9]{1,2}$' && { echo "thermal pressure"; return 0; }
    return 1
}

trap 'kill -TERM $(tree | cut -d" " -f2-) 2>/dev/null; exit 130' INT TERM

limit_kb=$((${CAPPED_MEM_GB:-12} * 1048576))
peak=0
start=$(date +%s)
stopped=
n=0
while kill -0 "$child" 2>/dev/null; do
    sleep 1
    n=$((n + 1))
    [ $((n % 5)) -eq 0 ] || continue
    t=$(tree)
    [ -n "$t" ] || break
    kb=${t%% *}
    pids=${t#* }
    [ "$kb" -gt "$peak" ] && peak=$kb
    if [ "$kb" -gt "$limit_kb" ]; then
        echo "capped: killed: the tree used $((kb / 1024)) MB, over CAPPED_MEM_GB=${CAPPED_MEM_GB:-12}" >&2
        kill -TERM $pids 2>/dev/null
        sleep 3
        kill -KILL $pids 2>/dev/null
        wait "$child" 2>/dev/null
        exit 137
    fi
    why=$(pressure)
    if [ -n "$why" ] && [ -z "$stopped" ]; then
        echo "capped: stopped while the Mac reports $why" >&2
        kill -STOP $pids 2>/dev/null
        stopped=1
    elif [ -z "$why" ] && [ -n "$stopped" ]; then
        echo "capped: resumed" >&2
        kill -CONT $pids 2>/dev/null
        stopped=
    fi
done
wait "$child"
rc=$?
echo "capped: exit $rc, peak $((peak / 1024)) MB, $(($(date +%s) - start)) s" >&2
exit $rc
