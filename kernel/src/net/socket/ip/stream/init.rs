// SPDX-License-Identifier: MPL-2.0

use aster_bigtcp::{
    socket::{RawTcpSetOption, UnboundTcpSocket},
    wire::IpEndpoint,
};

use super::{connecting::ConnectingStream, listen::ListenStream, StreamObserver};
use crate::{
    events::IoEvents,
    net::{
        iface::BoundTcpSocket,
        socket::ip::common::{bind_socket, get_ephemeral_endpoint},
    },
    prelude::*,
    process::signal::Pollee,
};

pub enum InitStream {
    Unbound(Box<UnboundTcpSocket>),
    Bound(BoundTcpSocket),
}

impl InitStream {
    pub fn new() -> Self {
        InitStream::Unbound(Box::new(UnboundTcpSocket::new()))
    }

    pub fn new_bound(bound_socket: BoundTcpSocket) -> Self {
        InitStream::Bound(bound_socket)
    }

    pub fn bind(
        self,
        endpoint: &IpEndpoint,
        can_reuse: bool,
        observer: StreamObserver,
    ) -> core::result::Result<BoundTcpSocket, (Error, Self)> {
        let unbound_socket = match self {
            InitStream::Unbound(unbound_socket) => unbound_socket,
            InitStream::Bound(bound_socket) => {
                return Err((
                    Error::with_message(Errno::EINVAL, "the socket is already bound to an address"),
                    InitStream::Bound(bound_socket),
                ));
            }
        };
        let bound_socket = match bind_socket(
            unbound_socket,
            endpoint,
            can_reuse,
            |iface, socket, config| iface.bind_tcp(socket, observer, config),
        ) {
            Ok(bound_socket) => bound_socket,
            Err((err, unbound_socket)) => return Err((err, InitStream::Unbound(unbound_socket))),
        };
        Ok(bound_socket)
    }

    fn bind_to_ephemeral_endpoint(
        self,
        remote_endpoint: &IpEndpoint,
        observer: StreamObserver,
    ) -> core::result::Result<BoundTcpSocket, (Error, Self)> {
        let endpoint = get_ephemeral_endpoint(remote_endpoint);
        self.bind(&endpoint, false, observer)
    }

    pub fn connect(
        self,
        remote_endpoint: &IpEndpoint,
        pollee: &Pollee,
    ) -> core::result::Result<ConnectingStream, (Error, Self)> {
        let bound_socket = match self {
            InitStream::Bound(bound_socket) => bound_socket,
            InitStream::Unbound(_) => self
                .bind_to_ephemeral_endpoint(remote_endpoint, StreamObserver::new(pollee.clone()))?,
        };

        ConnectingStream::new(bound_socket, *remote_endpoint)
            .map_err(|(err, bound_socket)| (err, InitStream::Bound(bound_socket)))
    }

    pub fn listen(
        self,
        backlog: usize,
        pollee: &Pollee,
    ) -> core::result::Result<ListenStream, (Error, Self)> {
        let InitStream::Bound(bound_socket) = self else {
            // FIXME: The socket should be bound to INADDR_ANY (i.e., 0.0.0.0) with an ephemeral
            // port. However, INADDR_ANY is not yet supported, so we need to return an error first.
            debug_assert!(false, "listen() without bind() is not implemented");
            return Err((
                Error::with_message(Errno::EINVAL, "listen() without bind() is not implemented"),
                self,
            ));
        };

        ListenStream::new(bound_socket, backlog, pollee)
            .map_err(|(err, bound_socket)| (err, InitStream::Bound(bound_socket)))
    }

    pub fn local_endpoint(&self) -> Option<IpEndpoint> {
        match self {
            InitStream::Unbound(_) => None,
            InitStream::Bound(bound_socket) => Some(bound_socket.local_endpoint().unwrap()),
        }
    }

    pub(super) fn check_io_events(&self) -> IoEvents {
        // Linux adds OUT and HUP events for a newly created socket
        IoEvents::OUT | IoEvents::HUP
    }

    pub(super) fn set_raw_option<R>(
        &mut self,
        set_option: impl Fn(&mut dyn RawTcpSetOption) -> R,
    ) -> R {
        match self {
            InitStream::Unbound(unbound_socket) => set_option(unbound_socket.as_mut()),
            InitStream::Bound(bound_socket) => set_option(bound_socket),
        }
    }
}
