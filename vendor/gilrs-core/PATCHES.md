# Local hot-plug patch

Upstream: gilrs-core 0.6.8, gilrs-project/gilrs commit
07e286e24b046cf39e5c367daa2770b805a64692, path gilrs-core.
Original dual Apache-2.0/MIT licensing is retained.

Bug 14: Windows Gaming Input can lose a controller between enumeration and
reading. Failed initial reads now skip that poll; failed updates do not emit
stale changes; raw axis/button discovery propagates the Windows error; late
connection callbacks tolerate a closed receiver. Windows is pinned to the
workspace's existing 0.62.2 dependency. Other platforms are unchanged.

Runtime discovery now also reconciles the enumerated WGI controller list with
the previous poll. It emits missing connect/disconnect events if a callback was
lost, retains the controller ID for unplug events after the device handle stops
answering, refreshes the WinRT handle when the same controller reconnects, and
ignores duplicate callback/poll transitions. A failed enumeration does not
disconnect every controller.

Each discovery sends a complete initial control snapshot; disconnected reading
buffers are discarded before reconnect. This initializes neutral raw axes as
well as held buttons, rather than retaining their default or pre-unplug values.
Connection callbacks invalidate snapshots even if remove/add occurs between polls.
Input for unknown or disconnected devices cannot create an implicit connection.
Ignored duplicate callbacks no longer terminate the nonblocking event drain.

Keep this patch until the corresponding fallible WGI paths are present in an
upstream release. Physical USB/Bluetooth hot-plug acceptance remains required.
