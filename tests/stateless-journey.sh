#!/usr/bin/env bash
set -eu

exe=${1:?First argument must be the executable to test}

root="$(cd "${0%/*}" && pwd)"
# shellcheck disable=1090
source "$root/utilities.sh"
snapshot="$root/snapshots"
fixture="$root/fixtures"

SUCCESSFULLY=0
WITH_FAILURE=1

(with "no input file"
  it "fails with an error message" && {
    WITH_SNAPSHOT="$snapshot/failure-missing-input-file" \
    expect_run ${WITH_FAILURE} "$exe"
  }
)

(with "an input file that does not exit"
  it "fails with an error message" && {
    WITH_SNAPSHOT="$snapshot/failure-input-file-not-found" \
    expect_run ${WITH_FAILURE} "$exe" some-file-that-does-not-exist
  }
)

(with "hello world piet image"
  it "produces the expected output" && {
    WITH_SNAPSHOT="$snapshot/success-input-file-produces-correct-output" \
    expect_run ${SUCCESSFULLY} "$exe" "$fixture/input.txt"
  }
)
