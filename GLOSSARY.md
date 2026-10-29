# Glossary

**Pad**: The USB macro pad easy-vial listens to, picked by device name. While the Daemon runs, the pad's keys no longer type into other apps. _Avoid_: keyboard, device (when you mean this specific pad).

**Slot**: One of the 9 numbered positions in the grid, made up of a learned key and a script. Either half may be unset, and a physical key belongs to at most one Slot. _Avoid_: binding, macro, key (when you mean the whole pairing).

**Daemon**: The always-running background listener that runs a Slot's script when its key is pressed. It keeps running after the window is closed. _Avoid_: service, listener.

**Learn**: Assigning a physical key to a Slot by pressing it on the Pad. _Avoid_: record, bind.
