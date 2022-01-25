FROM gitpod/workspace-full

USER gitpod

# The frontend is using elm, this is not included in workspace-full
RUN bash -cl "npm install -g elm elm-live elm-format swc"

# To live-restart the backend, we are using cargo-watch
RUN bash -cl "cargo install cargo-watch"