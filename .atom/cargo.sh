#!/bin/sh

# This script is used by .atom-build.json to run `cargo` for the Rust project associated
# with the file that is currently active in the Atom editor.  It walks up the directory
# tree until it finds a Cargo.toml file and then invokes `cargo` in that directory.
# For example, if frustum/frustum_core/src/signal.rs is the active file in Atom, and
# you use the "Build: Trigger [test]" command, it will run the frustum_core tests from
# inside Atom.

PROJECT_PATH=$1
ACTIVE_FILE_PATH=$2

shift
shift
ARGS="$@"

DIR=$ACTIVE_FILE_PATH
while [ -z $TOML_DIR ]
do
  if [ -f $DIR/Cargo.toml ]; then
    TOML_DIR=$DIR
  else
    # TODO: Ideally we'd require ACTIVE_FILE_PATH to be a subdirectory of PROJECT_PATH,
    # but for now we just keep walking up and terminate if we hit root
    if [ $DIR = $PROJECT_PATH -o $DIR = "/" ]; then
      echo "No Cargo.toml found for the current context"
      exit 1
    fi
    DIR=$(dirname $DIR)
  fi
done

cd $TOML_DIR
cargo $ARGS
