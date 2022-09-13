# Usage

In order to run the wrapper, **Ayudame** needs to be installed on the system, and `LD_LIBRARY_PATH` needs to point to the Ayudame's installation directory.

It is recommended to start the wrapper via a front end like **Temanejo**. In order to do so, compile the wrapper via `cargo build --release` and select the resulting binary and ayudame.lib in Temanejo.

When the wrapper is started, it will automatically send the `pre_init` and `init` events. At the moment, in order to change this, you will need to comment out lines 68 and 69 in `main.rs`. The issue, when sending `pre_init` and `init` manually, is that Temanejo will time out after a short while and abort, if those events are not sent fast enough.

Afterwards it is possible to send all supported events and interact with the frontend.