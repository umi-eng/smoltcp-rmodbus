use crate::PORT;
use rmodbus::{
    server::{context::ModbusContext, ModbusFrame},
    ErrorKind, ModbusFrameBuf, ModbusProto,
};
use smoltcp::{
    iface::{SocketHandle, SocketSet},
    socket::tcp::{
        ListenError, RecvError, SendError, Socket, SocketBuffer, State,
    },
    time::Duration,
};

/// Error type.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub enum Error {
    /// Receive buffer too small.
    RxBufferTooSmall,
    /// Transmit buffer too small.
    TxBufferTooSmall,
    /// Socket receive error.
    Receive(RecvError),
    /// Socket transmit error.
    Send(SendError),
    /// Socket listen error.
    Listen(ListenError),
    /// Modbus error.
    Modbus(ErrorKind),
}

/// Modbus server.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Server<CTX: ModbusContext> {
    handle: SocketHandle,
    context: CTX,
}

impl<CTX: ModbusContext> Server<CTX> {
    /// Creates a new server instance.
    pub fn new<'a>(
        sockets: &mut SocketSet<'a>,
        rx_buffer: SocketBuffer<'a>,
        tx_buffer: SocketBuffer<'a>,
        context: CTX,
    ) -> Result<Self, Error> {
        if rx_buffer.capacity() < 260 {
            return Err(Error::RxBufferTooSmall);
        }

        if tx_buffer.capacity() < 260 {
            return Err(Error::TxBufferTooSmall);
        }

        let mut socket = Socket::new(rx_buffer, tx_buffer);

        socket.set_timeout(Some(Duration::from_secs(10)));

        let handle = sockets.add(socket);

        Ok(Self { handle, context })
    }

    /// Process socket data.
    ///
    /// Returns `true` if the context was mutated.
    pub fn poll(&mut self, sockets: &mut SocketSet) -> Result<bool, Error> {
        let mut ctx_mutated = false;

        let socket = sockets.get_mut::<Socket>(self.handle);

        if !socket.is_open() {
            if !socket.is_listening() {
                socket.listen(PORT).map_err(|err| Error::Listen(err))?;
            }
        }

        if socket.state() == State::CloseWait {
            socket.close();
        }

        if socket.can_recv() && socket.can_recv() {
            let mut buf: ModbusFrameBuf = [0; 256];

            let len = socket
                .recv_slice(&mut buf)
                .map_err(|err| Error::Receive(err))?;

            if len == 0 {
                return Ok(ctx_mutated); // no bytes received
            }

            let mut response = heapless::Vec::<u8, 256>::new();

            let mut frame =
                ModbusFrame::new(1, &buf, ModbusProto::TcpUdp, &mut response);

            frame.parse().map_err(|err| Error::Modbus(err))?;

            if frame.processing_required {
                let result = if frame.readonly {
                    frame.process_read(&self.context)
                } else {
                    ctx_mutated = true;
                    frame.process_write(&mut self.context)
                };

                result.map_err(|err| Error::Modbus(err))?;
            }

            if frame.response_required {
                frame.finalize_response().unwrap();
                socket
                    .send_slice(response.as_slice())
                    .map_err(|err| Error::Send(err))?;
            }
        }

        Ok(ctx_mutated)
    }

    /// Modbus context.
    pub fn context(&self) -> &CTX {
        &self.context
    }

    /// Mutable Modbus context.
    pub fn context_mut(&mut self) -> &mut CTX {
        &mut self.context
    }
}

#[cfg(test)]
mod tests {
    use rmodbus::server::storage::ModbusStorageSmall;
    use smoltcp::iface::SocketStorage;

    use super::*;

    #[test]
    fn create_instance() {
        let mut socket_storage = [SocketStorage::EMPTY; 8];
        let mut socketset =
            SocketSet::new(&mut socket_storage.as_mut_slice()[..]);
        let mut rx_buf = [0; 260];
        let rx = SocketBuffer::new(&mut rx_buf.as_mut_slice()[..]);
        let mut tx_buf = [0; 260];
        let tx = SocketBuffer::new(&mut tx_buf.as_mut_slice()[..]);

        let context = ModbusStorageSmall::new();

        let mut server = Server::new(&mut socketset, rx, tx, context).unwrap();

        server.poll(&mut socketset).ok();
    }

    #[test]
    #[should_panic]
    fn buffer_too_small() {
        let mut socket_storage = [SocketStorage::EMPTY; 8];
        let mut socketset =
            SocketSet::new(&mut socket_storage.as_mut_slice()[..]);
        let mut rx_buf = [0; 259];
        let rx = SocketBuffer::new(&mut rx_buf.as_mut_slice()[..]);
        let mut tx_buf = [0; 260];
        let tx = SocketBuffer::new(&mut tx_buf.as_mut_slice()[..]);

        let context = ModbusStorageSmall::new();

        let server = Server::new(&mut socketset, rx, tx, context).unwrap();
    }
}
