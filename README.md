This crate provides general mechanisms for implementing traits on types by
forwarding an implementation provided by another type.

Two different forwarding methods are provided: Forwarding traits implemented by
members, and forwarding traits implemented by types that the receiver type can
convert to.  These methods may be used in combination on the same receiver type.
This crate fully supports generic traits and struct types.

See crate documentation for more details.
