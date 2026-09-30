#!/usr/bin/env bash
# Measures what the watcher costs in a container: CPU time and memory of its one process, read from /proc.
#
#   scripts/measure.sh idle            resting memory, and CPU over a minute
#   scripts/measure.sh start N         a source of N folders of 10 files, changed lately: what a start costs and holds
#   scripts/measure.sh batch N         N folders of 10 files, moved as N batches
#   scripts/measure.sh single N        N folders of 10 files, moved as one batch
#   scripts/measure.sh rounds N        N folders of 10 files arriving while it runs, five times over
#   scripts/measure.sh copy GIB        one file of GIB gibibytes moved across filesystems
#   scripts/measure.sh all             the scenarios the promises of docs/design.md 4.6 were measured with
#
# It needs docker and nothing else, so it runs on the machine the watcher is meant for. The image is
# built from this repository unless AUTO_RENAMER_IMAGE names one to measure instead, and AUTO_RENAMER_MEMORY
# gives the container a memory limit to try, such as 64m. CPU time depends on the machine; memory does
# not. `rounds` fails when a file is left in the source or a batch is handed over twice, which no
# machine should show.
set -euo pipefail

cd "$(dirname "$0")/.."

IMAGE=${AUTO_RENAMER_IMAGE:-auto-renamer:measure}
PREFIX=auto-renamer-measure
RUN=$PREFIX-run
PROBE=$PREFIX-probe
FILES_PER_FOLDER=10

PIPELINE='
[pipeline.video]
stages = [
  { filter = { ext = ["mkv", "mp4"] } },
  { number = { into = "episode" } },
  { default = { season = 1 } },
  { format = "{show} s{season:02}e{episode:02}" },
  "move",
  "cleanup",
]
'

# The comment this file opens with is what it says of itself.
usage() {
  awk 'NR > 1 && /^#/ { sub(/^# ?/, ""); print; next } NR > 1 { exit }' "$0"
  exit 2
}

clean() {
  docker rm -f "$RUN" "$PROBE" > /dev/null 2>&1 || true
  docker volume rm -f "$PREFIX-config" "$PREFIX-data" "$PREFIX-source" "$PREFIX-target" > /dev/null 2>&1 || true
}

# The probe shares the process table of the host, so it can read /proc of the watcher, whose own image
# holds nothing to read it with.
prepare() {
  trap clean EXIT
  clean
  if [ -z "${AUTO_RENAMER_IMAGE:-}" ]; then
    docker build -q -t "$IMAGE" . > /dev/null
  fi
  docker run -d --name "$PROBE" --pid=host --cap-add SYS_PTRACE alpine sleep infinity > /dev/null
  docker volume create "$PREFIX-config" > /dev/null
}

# Writes the configuration: the watch reads its settings from standard input.
configure() {
  { echo "$PIPELINE"; echo "[watch.measured]"; cat; } |
    docker run --rm -i -v "$PREFIX-config:/config" alpine sh -c 'cat > /config/config.toml'
}

# Makes folders of files in a volume: volume, folder inside it, how many folders.
folders() {
  docker run --rm -v "$1:/volume" alpine sh -c '
    folder=1
    while [ $folder -le '"$3"' ]; do
      path=$(printf "/volume/'"$2"'/T%05d" $folder); mkdir -p "$path"
      file=1
      while [ $file -le '"$FILES_PER_FOLDER"' ]; do
        : > "$path/[Rel] Alpha - $(printf %02d $file) [1080p].mkv"
        file=$((file + 1))
      done
      folder=$((folder + 1))
    done'
}

start() {
  local limit=()
  [ -n "${AUTO_RENAMER_MEMORY:-}" ] && limit=(--memory "$AUTO_RENAMER_MEMORY" --memory-swap "$AUTO_RENAMER_MEMORY")
  docker run -d --name "$RUN" -v "$PREFIX-config:/etc/auto-renamer" ${limit[@]+"${limit[@]}"} "$@" "$IMAGE" > /dev/null
  PID=$(docker inspect -f '{{.State.Pid}}' "$RUN")
}

# Milliseconds of CPU the watcher has used, user and system, at 100 ticks a second.
cpu_ms() {
  docker exec "$PROBE" awk '{ print ($14 + $15) * 10 }' "/proc/$PID/stat"
}

# Kilobytes of memory: `VmRSS` for what it holds now, `VmHWM` for the most it ever held.
memory_kb() {
  docker exec "$PROBE" awk -v field="$1:" '$1 == field { print $2 }' "/proc/$PID/status"
}

moved() {
  docker logs "$RUN" 2>&1 | grep -c '^\[info\] .* -> ' || true
}

# Waits until a number of files were moved, and ends the measurement when two minutes were not enough.
until_moved() {
  local waited=0
  while [ "$(moved)" -lt "$1" ]; do
    still_running
    if [ $waited -ge 120 ]; then
      echo "only $(moved) of $1 files were moved in two minutes" >&2
      exit 1
    fi
    sleep 1
    waited=$((waited + 1))
  done
}

# Waits until the CPU used in two seconds stops changing: three samples in a row close to one another.
until_steady() {
  local before after used first=-1 second=-1 low high
  before=$(cpu_ms)
  while :; do
    sleep 2
    after=$(cpu_ms)
    used=$((after - before))
    before=$after
    if [ $second -ge 0 ]; then
      low=$used
      high=$used
      [ $first -lt $low ] && low=$first
      [ $second -lt $low ] && low=$second
      [ $first -gt $high ] && high=$first
      [ $second -gt $high ] && high=$second
      [ $((high - low)) -le $((high / 4 + 20)) ] && break
    fi
    second=$first
    first=$used
  done
}

# Ends the measurement when the watcher is no longer running, saying whether the kernel ended it for
# memory, which is what a limit set too low shows as.
still_running() {
  [ "$(docker inspect -f '{{.State.Running}}' "$RUN")" = true ] && return
  echo "the watcher stopped: $(docker inspect -f 'out_of_memory={{.State.OOMKilled}} exit_code={{.State.ExitCode}}' "$RUN")" >&2
  exit 1
}

report() {
  still_running
  echo "$* now_kb=$(memory_kb VmRSS) peak_kb=$(memory_kb VmHWM)"
}

idle() {
  prepare
  docker volume create "$PREFIX-source" > /dev/null
  docker volume create "$PREFIX-target" > /dev/null
  configure << CONFIG
source = "/source"
target = "/target"
pipelines = ["video"]
CONFIG
  start -v "$PREFIX-source:/source" -v "$PREFIX-target:/target"
  sleep 5
  local before
  before=$(cpu_ms)
  sleep 60
  report "idle cpu_ms_per_minute=$(($(cpu_ms) - before))"
}

# The files were changed lately and the window is an hour, so every one of them is held.
start_with() {
  prepare
  docker volume create "$PREFIX-data" > /dev/null
  folders "$PREFIX-data" source "$1"
  configure << CONFIG
source = "/data/source"
target = "/data/target"
pipelines = ["video"]
vars = { show = "Alpha" }
batch_window = "1h"
batch_max_wait = "1h"
CONFIG
  start -v "$PREFIX-data:/data"
  until_steady
  local started before
  started=$(cpu_ms)
  before=$started
  sleep 30
  report "start folders=$1 start_cpu_ms=$started held_cpu_ms_per_minute=$((($(cpu_ms) - before) * 2))"
}

batch() {
  prepare
  docker volume create "$PREFIX-data" > /dev/null
  folders "$PREFIX-data" source "$1"
  sleep 3
  configure << CONFIG
source = "/data/source"
target = "/data/target"
pipelines = ["video"]
vars = { show = "Alpha" }
batch_window = "2s"
batch_max_wait = "2s"
CONFIG
  start -v "$PREFIX-data:/data"
  until_moved $(($1 * FILES_PER_FOLDER))
  sleep 5
  report "batch folders=$1 moved=$(moved) cpu_ms=$(cpu_ms)"
}

# More files leave the source in one turn than the queue of notifications has room for, so this is
# also what starting over after lost notifications costs.
single() {
  prepare
  docker volume create "$PREFIX-data" > /dev/null
  folders "$PREFIX-data" source "$1"
  sleep 3
  configure << CONFIG
source = "/data/source"
target = "/data/target"
pipelines = ["video"]
unit = "source"
vars = { show = "Alpha" }
batch_window = "2s"
batch_max_wait = "2s"
batch_max = $(($1 * FILES_PER_FOLDER))
CONFIG
  start -v "$PREFIX-data:/data"
  until_moved $(($1 * FILES_PER_FOLDER))
  sleep 5
  report "single files=$(($1 * FILES_PER_FOLDER)) moved=$(moved) cpu_ms=$(cpu_ms) started_over=$(docker logs "$RUN" 2>&1 | grep -c 'notifications were lost' || true)"
}

rounds() {
  prepare
  docker volume create "$PREFIX-data" > /dev/null
  docker run --rm -v "$PREFIX-data:/data" alpine mkdir -p /data/source
  configure << CONFIG
source = "/data/source"
target = "/data/target"
pipelines = ["video"]
vars = { show = "Alpha" }
batch_window = "2s"
batch_max_wait = "2s"
CONFIG
  start -v "$PREFIX-data:/data"
  sleep 3
  report "rounds start"
  local round left twice
  for round in 1 2 3 4 5; do
    folders "$PREFIX-data" "source/R$round" "$1"
    until_moved $((round * $1 * FILES_PER_FOLDER))
    sleep 8
    report "rounds round=$round moved=$(moved) cpu_ms=$(cpu_ms)"
  done
  left=$(docker run --rm -v "$PREFIX-data:/data" alpine sh -c 'find /data/source -type f | wc -l')
  twice=$(docker logs "$RUN" 2>&1 | grep -c 'skipped: missing' || true)
  echo "rounds folders=$1 left_in_source=$left handed_over_twice=$twice"
  [ "$left" -eq 0 ] && [ "$twice" -eq 0 ]
}

# The source is kept in memory and the target on disk, so the move has to copy.
copy() {
  prepare
  docker volume create --driver local --opt type=tmpfs --opt device=tmpfs \
    --opt "o=size=$(($1 + 1))g" "$PREFIX-source" > /dev/null
  docker volume create "$PREFIX-target" > /dev/null
  configure << CONFIG
source = "/source"
target = "/target"
pipelines = ["video"]
vars = { show = "Alpha" }
batch_window = "6s"
batch_max_wait = "6s"
CONFIG
  start -v "$PREFIX-source:/source" -v "$PREFIX-target:/target"
  sleep 2
  docker run --rm -v "$PREFIX-source:/source" alpine sh -c \
    "mkdir -p /source/Alpha && dd if=/dev/urandom of='/source/Alpha/[Rel] Alpha - 01 [1080p].mkv' bs=1M count=$(($1 * 1024)) 2> /dev/null"
  still_running
  local written
  written=$(cpu_ms)
  until_moved 1
  still_running
  report "copy gib=$1 moved=$(moved) write_cpu_ms=$written copy_cpu_ms=$(($(cpu_ms) - written))"
}

case "${1:-}" in
  idle) idle ;;
  start) start_with "${2:?how many folders}" ;;
  batch) batch "${2:?how many folders}" ;;
  single) single "${2:?how many folders}" ;;
  rounds) rounds "${2:?how many folders}" ;;
  copy) copy "${2:?how many gibibytes}" ;;
  all)
    idle
    for size in 100 1000 10000; do start_with $size; done
    batch 10000
    single 1000
    rounds 1000
    copy 2
    ;;
  *) usage ;;
esac
